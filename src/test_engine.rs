use std::path::Path;

use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::ledger::*;
use radix_engine::transaction::{execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt, TransactionResult};
use radix_engine::types::*;
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};

use radix_engine_interface::{dec, rule};
use radix_engine_interface::api::component::ComponentStateSubstate;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::node_modules::metadata::MetadataEntry;
use radix_engine_interface::api::types::{RENodeId, VaultOffset};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::FAUCET_COMPONENT;
use radix_engine_interface::math::Decimal;

use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::{Executable, TestTransaction};
use transaction::model::TransactionManifest;

use crate::account::Account;
use crate::compiler::compile;
use crate::state_hash::StateHashSupport;

pub struct TestEngine {
    next_private_key: u64,
    next_transaction_nonce: u64,
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    state_hash_support: StateHashSupport,
    sub_state_store: TypedInMemorySubstateStore,
}

impl TestEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            next_private_key: 1,
            next_transaction_nonce: 0,
            scrypto_interpreter: ScryptoInterpreter {
                wasm_metering_config: WasmMeteringConfig::V0,
                wasm_engine: DefaultWasmEngine::default(),
                wasm_instrumenter: WasmInstrumenter::default(),
            },
            state_hash_support: StateHashSupport::new(),
            sub_state_store: TypedInMemorySubstateStore::new()
        };


        let genesis =  create_genesis(BTreeMap::new(), BTreeMap::new(), 1u64, 1u64, 1u64);
        let receipt = engine.execute_transaction(
            genesis.get_executable(vec![AuthAddresses::system_role()]),
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
        );
        receipt.expect_commit_success();
        engine.generate_initial_validators(10);
        engine
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleGlobalId>,
        with_trace: bool
    ) -> TransactionReceipt {

        let transaction = TestTransaction::new(manifest, self.next_transaction_nonce(), u32::MAX);
        let executable = transaction.get_executable(initial_proofs);
        let fee_reserve_config = FeeReserveConfig::default();
        let execution_config = ExecutionConfig::default().with_trace(with_trace);

        let transaction_receipt = self.execute_transaction(
            executable,
            &fee_reserve_config,
            &execution_config,
        );

        transaction_receipt
    }

    pub fn execute_transaction(
        &mut self,
        executable: Executable,
        fee_reserve_config: &FeeReserveConfig,
        execution_config: &ExecutionConfig,
    ) -> TransactionReceipt {

        let receipt = execute_transaction(
            &mut self.sub_state_store,
            &self.scrypto_interpreter,
            fee_reserve_config,
            execution_config,
            &executable
        );

        if let TransactionResult::Commit(commit) = &receipt.result {
            let commit_receipt = commit.state_updates.commit(&mut self.sub_state_store);
            self.state_hash_support.update_with(commit_receipt.outputs);
        };

        receipt
    }

    pub fn get_balance_of(
        &self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Decimal {
        let node_id = RENodeId::GlobalObject(component_address.into());
        let mut vault_finder = VaultFinder::new(resource_address);

        let mut state_tree_visitor =
            StateTreeTraverser::new(&self.sub_state_store, &mut vault_finder, 100);
        state_tree_visitor
            .traverse_all_descendents(None, node_id)
            .unwrap();

        vault_finder
            .to_vaults()
            .get(0)
            .map_or(Decimal::zero(), |vault_id| self.get_vault_balance(vault_id))
    }

    pub fn get_component_state<T: ScryptoDecode>(
        &self,
        component_address: &ComponentAddress,
    ) -> T {
        let component_state: ComponentStateSubstate = self.sub_state_store
            .get_substate(&SubstateId(
                RENodeId::GlobalObject(Address::Component(component_address.clone())),
                NodeModuleId::SELF,
                SubstateOffset::Component(ComponentOffset::State0),
            ))
            .map(|s| s.substate.to_runtime())
            .map(|s| s.into())
            .unwrap();
        let raw_state = IndexedScryptoValue::from_scrypto_value(component_state.0);
        raw_state.as_typed::<T>().unwrap()
    }

    pub fn get_metadata(&self, address: Address, key: &str) -> Option<MetadataEntry> {
        let metadata_entry = self
            .sub_state_store
            .get_substate(&SubstateId(
                address.into(),
                NodeModuleId::Metadata,
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                    scrypto_encode(key).unwrap(),
                )),
            ))
            .map(|s| s.substate.to_runtime())?;

        let metadata_entry: Option<ScryptoValue> = metadata_entry.into();
        let metadata_entry = match metadata_entry {
            Some(value) => {
                let value: MetadataEntry =
                    scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap();
                Some(value)
            }
            None => None,
        };

        metadata_entry
    }

    pub fn new_account(&mut self) -> Account {
        let (public_key, private_key) = self.new_key_pair();
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, dec!(100))
            .new_account(rule!(require(NonFungibleGlobalId::from_public_key(
                &public_key
            ))))
            .build();

        let receipt = self.execute_manifest(manifest, vec![], false);
        let account_component = receipt.expect_commit(true).new_component_addresses()[0];

        Account::new(private_key, public_key, account_component)
    }

    pub fn publish_package<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        let (code, schema) = compile(package_dir);
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .publish_package(
                code,
                schema,
                BTreeMap::new(),
                BTreeMap::new(),
                AccessRulesConfig::new(),
            )
            .build();

        let receipt = self.execute_manifest(manifest, vec![], false);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    pub fn publish_package_with_owner<P: AsRef<Path>>(
        &mut self,
        package_dir: P,
        owner_badge: NonFungibleGlobalId,
    ) -> PackageAddress {
        let (code, schema) = compile(package_dir);
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100u32.into())
            .publish_package_with_owner(code, schema, owner_badge)
            .build();

        let receipt = self.execute_manifest(manifest, vec![], false);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    fn get_vault_balance(&self, vault_id: &ObjectId) -> Decimal {
        if let Some(output) = self.sub_state_store.get_substate(&SubstateId(
            RENodeId::Object(vault_id.clone()),
            NodeModuleId::SELF,
            SubstateOffset::Vault(VaultOffset::Info),
        )) {
            if output.substate.vault_info().resource_type.is_fungible() {
                self.sub_state_store
                    .get_substate(&SubstateId(
                        RENodeId::Object(vault_id.clone()),
                        NodeModuleId::SELF,
                        SubstateOffset::Vault(VaultOffset::LiquidFungible),
                    ))
                    .map(|mut output| output.substate.vault_liquid_fungible_mut().amount())
                    .unwrap_or(Decimal::zero())
            } else {
                self.sub_state_store
                    .get_substate(&SubstateId(
                        RENodeId::Object(vault_id.clone()),
                        NodeModuleId::SELF,
                        SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                    ))
                    .map(|mut output| {
                        output
                            .substate
                            .vault_liquid_non_fungible_mut()
                            .ids()
                            .clone()
                    })
                    .map(|ids| ids.len().into()).unwrap_or(Decimal::zero())
            }
        } else {
            Decimal::zero()
        }
    }

    fn new_key_pair(&mut self) -> (EcdsaSecp256k1PublicKey, EcdsaSecp256k1PrivateKey) {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();
        (public_key, private_key)
    }
    fn next_transaction_nonce(&mut self) -> u64 {
        self.next_transaction_nonce += 1;
        self.next_transaction_nonce - 1
    }

    fn next_private_key(&mut self) -> u64 {
        self.next_private_key += 1;
        self.next_private_key - 1
    }

    fn generate_initial_validators(&mut self, amount: usize) {
        for _ in 0..amount {
            let (pub_key, _) = self.new_key_pair();
            let non_fungible_id = NonFungibleGlobalId::from_public_key(&pub_key);
            let manifest = ManifestBuilder::new()
                .lock_fee(FAUCET_COMPONENT, 10.into())
                .create_validator(pub_key, rule!(require(non_fungible_id)))
                .build();
            let receipt = self.execute_manifest(manifest, vec![], false);
            receipt.expect_commit(true);
        }
    }
}


