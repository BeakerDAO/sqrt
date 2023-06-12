use radix_engine::transaction::{TransactionOutcome, TransactionReceipt, TransactionResult};
use radix_engine::types::{count, dec, manifest_decode, ComponentAddress, Decimal, Encoder, ManifestEncoder, ManifestValueKind, PackageAddress, MANIFEST_SBOR_V1_MAX_DEPTH, MANIFEST_SBOR_V1_PAYLOAD_PREFIX, ManifestExpression};
use radix_engine_interface::constants::FAUCET_COMPONENT;
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;

use std::vec::Vec;

use crate::environment_encoder::EnvironmentEncode;
use crate::from_return::FromReturn;
use crate::manifest_args;
use crate::test_environment::TestEnvironment;
use crate::utils::get_outcome_and_fees;

#[derive(Clone)]
pub enum Outcome {
    /// States that transaction should
    Success,

    /// States that an assertion is expected to fail with a given message
    AssertionFailed(String),

    /// States that another error should happen
    OtherError(String),
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        match self {
            Outcome::Success => true,
            _ => false,
        }
    }
}

pub struct CallBuilder<'a> {
    caller: ComponentAddress,
    manifest: Option<TransactionManifest>,
    fee_locked: Decimal,
    fee_payer: ComponentAddress,
    test_environment: &'a mut TestEnvironment,
    with_trace: bool,
}

impl<'a> CallBuilder<'a> {
    pub fn from(test_env: &'a mut TestEnvironment, caller: ComponentAddress) -> Self {
        Self {
            caller: caller.clone() ,
            manifest: None,
            fee_locked: dec!(10),
            fee_payer: caller,
            test_environment: test_env,
            with_trace: false,
        }
    }

    pub fn lock(mut self, amount: Decimal) -> Self {
        self.fee_locked = amount;
        self
    }

    pub fn faucet_pays_fees(mut self) -> Self {
        self.fee_payer = FAUCET_COMPONENT;
        self
    }

    pub fn run(mut self) -> MethodCallReceipt {
        self.build();
        let receipt = self
            .test_environment
            .execute_call(self.manifest.clone().unwrap(), self.with_trace.clone());
        let (fee_paid, outcome) = get_outcome_and_fees(&receipt);
        MethodCallReceipt::from(fee_paid, outcome, receipt)
    }

    pub fn with_trace(mut self, trace: bool) -> Self {
        self.with_trace = trace;
        self
    }

    fn build(&mut self) {
        self.lock_fee();
        self.deposit_batch();
    }

    fn lock_fee(&mut self) {
        self.manifest.as_mut().unwrap().instructions.insert(
            0,
            transaction::model::Instruction::CallMethod {
                component_address: self.fee_payer,
                method_name: "lock_fee".to_string(),
                args: manifest_args!(self.fee_locked),
            },
        );
    }

    fn deposit_batch(&mut self) {
        self.manifest.as_mut().unwrap().instructions.push(
            transaction::model::Instruction::CallMethod {
                component_address: self.caller.clone(),
                method_name: "deposit_batch".to_string(),
                args: manifest_args!(ManifestExpression::EntireWorktop)
            }
        );
    }

    pub(crate) fn call_method(
        mut self,
        component: ComponentAddress,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {

        let mut manifest = ManifestBuilder::new();

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            arg.encode(
                &mut self.test_environment,
                &mut manifest,
                &mut encoder,
                self.caller.clone(),
            );
        }

        let manifest_arg = manifest_decode(&buf).unwrap();

        let transaction = manifest
            .call_method(component, method_name, manifest_arg)
            .build();
        self.manifest = Some(transaction);
        self
    }

    pub(crate) fn call_function(
        mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest = ManifestBuilder::new();

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            arg.encode(
                &mut self.test_environment,
                &mut manifest,
                &mut encoder,
                self.caller.clone(),
            );
        }

        let manifest_arg = manifest_decode(&buf).unwrap();

        let transaction = manifest
            .call_function(package_address, blueprint_name, function_name, manifest_arg)
            .build();
        self.manifest = Some(transaction);
        self
    }
}

pub struct MethodCallReceipt {
    fee_paid: Option<Decimal>,
    outcome: Outcome,
    receipt: TransactionReceipt,
}

impl MethodCallReceipt {
    pub fn from(fee_paid: Option<Decimal>, outcome: Outcome, receipt: TransactionReceipt) -> Self {
        Self {
            fee_paid,
            outcome,
            receipt,
        }
    }

    pub fn fee_paid(&self) -> Decimal {
        self.fee_paid.clone().unwrap()
    }

    pub fn get_return<T: FromReturn>(&self) -> Option<T> {
        match self.receipt().result {
            TransactionResult::Commit(commit) => match commit.outcome {
                TransactionOutcome::Success(commit_success) => Some(T::from(commit_success)),
                TransactionOutcome::Failure(_) => None,
            },
            _ => None,
        }
    }

    pub fn receipt(&self) -> TransactionReceipt {
        self.receipt.clone()
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome.clone()
    }
}