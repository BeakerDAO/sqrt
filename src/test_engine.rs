use crate::account::Account;
use crate::compiler::compile;
use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::ledger::{
    ReadableSubstateStore, StateTreeTraverser, TypedInMemorySubstateStore, VaultFinder,
};
use radix_engine::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt, TransactionResult,
};
use radix_engine::types::{ComponentAddress, Decimal, PackageAddress, ResourceAddress};
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine_interface::api::types::{
    NodeModuleId, ObjectId, RENodeId, SubstateId, SubstateOffset, VaultOffset,
};
use radix_engine_interface::blueprints::resource::{AccessRulesConfig, NonFungibleGlobalId};
use radix_engine_interface::constants::FAUCET_COMPONENT;
use radix_engine_interface::rule;
use std::collections::BTreeMap;
use std::path::Path;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::{TestTransaction, TransactionManifest};

pub struct TestEngine {
    scrypto_interpreter: ScryptoInterpreter<DefaultWasmEngine>,
    sub_state_store: TypedInMemorySubstateStore,
    next_private_key: u64,
    next_transaction_nonce: u64,
}

impl TestEngine {
    pub fn new() -> Self {
        Self {
            scrypto_interpreter: ScryptoInterpreter {
                wasm_metering_config: WasmMeteringConfig::V0,
                wasm_engine: DefaultWasmEngine::default(),
                wasm_instrumenter: WasmInstrumenter::default(),
            },
            sub_state_store: TypedInMemorySubstateStore::new(),
            next_private_key: 1,
            next_transaction_nonce: 0,
        }
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifest,
        initial_proofs: Vec<NonFungibleGlobalId>,
    ) -> TransactionReceipt {
        let transaction = TestTransaction::new(manifest, self.next_transaction_nonce(), -1);
        let executable = transaction.get_executable(initial_proofs);
        let fee_reserve_config = FeeReserveConfig::default();
        let execution_config = ExecutionConfig::default();

        let transaction_receipt = execute_transaction(
            &mut self.substate_store,
            &self.scrypto_interpreter,
            &fee_reserve_config,
            &execution_config,
            &executable,
        );

        if let TransactionResult::Commit(commit) = &transaction_receipt.result {
            let commit_receipt = commit.state_updates.commit(&mut self.substate_store);
            if let Some(state_hash_support) = &mut self.state_hash_support {
                state_hash_support.update_with(commit_receipt.outputs);
            }
        }

        transaction_receipt
    }

    pub fn get_balance_of(
        &self,
        component_address: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Decimal {
        let node_id = RENodeId::GlobalObject(component_address.into());
        let mut vault_finder = VaultFinder::new(resource_address);

        let mut state_tree_visitor =
            StateTreeTraverser::new(&self.substate_store, &mut vault_finder, 100);
        state_tree_visitor
            .traverse_all_descendents(None, node_id)
            .unwrap();

        vault_finder
            .to_vaults()
            .get(0)
            .map_or(Decimal::zero(), |vault_id| self.get_vault_balance(vault_id))
    }

    pub fn new_account(&mut self) -> Account {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(self.next_private_key()).unwrap();
        let public_key = private_key.public_key();
        let manifest = ManifestBuilder::new()
            .new_account(rule!(require(NonFungibleGlobalId::from_public_key(
                &key_pair.0
            ))))
            .build();

        let receipt = self.execute_manifest(manifest, vec![]);
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

        let receipt = self.execute_manifest(manifest, vec![]);
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

        let receipt = self.execute_manifest(manifest, vec![]);
        receipt.expect_commit(true).new_package_addresses()[0]
    }

    fn get_vault_balance(&self, vault_id: &ObjectId) -> Decimal {
        if let Some(output) = self.substate_store().get_substate(&SubstateId(
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
                self.substate_store()
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
                    .map(|ids| ids.len().into())
            }
        } else {
            Decimal::zero()
        }
    }

    fn next_transaction_nonce(&mut self) -> u64 {
        self.next_transaction_nonce += 1;
        self.next_transaction_nonce - 1
    }

    fn next_private_key(&mut self) -> u64 {
        self.next_private_key += 1;
        self.next_private_key - 1
    }
}
