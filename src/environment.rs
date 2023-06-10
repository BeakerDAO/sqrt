//! Defines methods that can be called for a blueprint

use std::collections::BTreeSet;
use std::vec::Vec;
use radix_engine::types::{ComponentAddress, count, Decimal, Encoder, ManifestEncoder, manifest_decode, MANIFEST_SBOR_V1_MAX_DEPTH, MANIFEST_SBOR_V1_PAYLOAD_PREFIX, ManifestValueKind, NonFungibleLocalId};
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;
use crate::environment_encoder::EnvironmentEncode;
use crate::formattable::Formattable;
use crate::manifest_args;
use crate::test_environment::TestEnvironment;

pub enum Environment<F: Formattable + Clone> {
    Account(F),
    Component(F),
    Package(F),
    FungibleBucket(F, Decimal),
    NonFungibleBucket(F, Vec<NonFungibleLocalId>),
    FungibleProof(F, Decimal),
    NonFungibleProof(F, Vec<NonFungibleLocalId>)
}

impl<F: Formattable + Clone> EnvironmentEncode for Environment<F> {
    fn encode(&self, test_environment: &TestEnvironment, manifest_builder: &mut ManifestBuilder, encoder: &mut ManifestEncoder, caller: ComponentAddress) {
        match self{
            Environment::Account(name) =>
                {
                    let account_address = test_environment.get_account(name.clone());
                    encoder.encode(&account_address).unwrap();
                }
            Environment::Component(name) =>
                {
                    let component_address = test_environment.get_component(name.clone());
                    encoder.encode(&component_address).unwrap();
                }
            Environment::Package(name) =>
                {
                    let package_address = test_environment.get_package(name.clone());
                    encoder.encode(&package_address).unwrap();
                }
            Environment::FungibleBucket(resource_name, amount) =>
                {
                    let resource_address = test_environment.get_fungible(resource_name.clone());
                    manifest_builder.call_method(caller, "withdraw", manifest_args!(resource_address.clone(), amount));
                    let (_, bucket, _) = manifest_builder.add_instruction(Instruction::TakeFromWorktopByAmount { amount: amount.clone(), resource_address });
                    encoder.encode(&(bucket.unwrap())).unwrap();

                }
            Environment::NonFungibleBucket(resource_name, ids) =>
                {
                    let resource_address = test_environment.get_fungible(resource_name.clone());
                    let set_ids = BTreeSet::from_iter(ids.iter().cloned());
                    manifest_builder.call_method(caller, "withdraw_by_ids", manifest_args!(resource_address.clone(), set_ids.clone()));
                    let (_, bucket, _) = manifest_builder.add_instruction(Instruction::TakeFromWorktopByIds { ids: set_ids, resource_address });
                    encoder.encode(&(bucket.unwrap())).unwrap();
                }
            Environment::FungibleProof(resource_name, amount) =>
                {
                    let resource_address = test_environment.get_fungible(resource_name.clone());
                    manifest_builder.call_method(caller, "create_proof_by_amount", manifest_args!(resource_address.clone(), amount));
                    let (_, _, proof) = manifest_builder.add_instruction(Instruction::CreateProofFromAuthZoneByAmount { amount: amount.clone(), resource_address });
                    encoder.encode(&(proof.unwrap())).unwrap();
                }
            Environment::NonFungibleProof(resource_name, ids) =>
                {
                    let resource_address = test_environment.get_fungible(resource_name.clone());
                    let set_ids = BTreeSet::from_iter(ids.iter().cloned());
                    manifest_builder.call_method(caller, "create_proof_by_ids", manifest_args!(resource_address.clone(), set_ids.clone()));
                    let (_, _, proof) = manifest_builder.add_instruction(Instruction::CreateProofFromAuthZoneByIds { ids: set_ids, resource_address });
                    encoder.encode(&(proof.unwrap())).unwrap();
                }
        }
    }
}