use radix_engine::errors::{KernelError, RejectionError, RuntimeError};
use radix_engine::transaction::{TransactionReceipt, TransactionResult};
use radix_engine::types::*;
use radix_engine::wasm::WasmRuntimeError;
use radix_engine_interface::constants::FAUCET_COMPONENT;
use radix_engine_interface::data::scrypto::ScryptoDecode;
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;
use crate::test_environment::TestEnvironment;

enum Outcome{
    /// States that transaction should
    Success,

    /// States that an assertion is expected to fail with a given message
    AssertionFailed(String),

    /// States that another error should happen
    OtherError(String),
}

impl Outcome {
}

pub struct MethodCallBuilder<'a>{
    caller: ComponentAddress,
    manifest: &'a ManifestBuilder,
    fee_locked: Decimal,
    fee_payer: ComponentAddress,
    test_environment: &'a TestEnvironment,
    with_trace: bool,
}

impl<'a> MethodCallBuilder<'a>{

    pub fn from(args: ManifestValue, caller: ComponentAddress, component: ComponentAddress, method_name: &'a str, test_env: &'a mut TestEnvironment) -> Self{
        let manifest = ManifestBuilder::new().call_method(component, method_name, args);
        Self{
            caller: caller.clone(),
            manifest,
            fee_locked: dec!(10),
            fee_payer: caller,
            test_environment: test_env,
            with_trace: false
        }
    }

    pub fn lock(mut self, amount: Decimal) -> Self {
        self.fee_locked = amount;
        self
    }

    pub fn faucet_pays_fees(mut self) -> Self{
        self.fee_payer = FAUCET_COMPONENT;
        self
    }

    pub fn run(mut self) -> MethodCallReceipt<()> {
        let manifest = self.to_manifest();
        let receipt = self.test_environment.execute_call(manifest, self.with_trace);
        let (fee_paid, outcome) = Self::get_outcome_and_fees(&receipt);
        MethodCallReceipt::from(fee_paid, None, outcome)
    }

    pub fn with_trace(mut self, trace: bool) -> Self{
        self.with_trace = trace;
        self
    }

    fn get_outcome_and_fees(receipt: &TransactionReceipt) -> (Option<Decimal>, Outcome)
    {
        match receipt.result.clone() {
            TransactionResult::Commit(commit_result) =>
                {
                    (Some(commit_result.fee_summary.total_execution_cost_xrd.clone()), Outcome::Success)
                }
            TransactionResult::Reject(rejected) =>
                {
                    let outcome = match rejected.error {
                        RejectionError::ErrorBeforeFeeLoanRepaid(error) =>
                            {
                                match error {
                                    RuntimeError::KernelError(kerror) =>
                                        {
                                            match kerror {
                                                KernelError::WasmRuntimeError(wasm_error) =>
                                                    {
                                                        match wasm_error{
                                                            WasmRuntimeError::InterpreterError(string_error) =>
                                                                {
                                                                    Outcome::AssertionFailed(string_error)
                                                                }
                                                            _ => Outcome::OtherError(rejected.error.to_string())
                                                        }
                                                    }
                                                _ => Outcome::OtherError(rejected.error.to_string())
                                            }
                                        }
                                    _ => Outcome::OtherError(rejected.error.to_string())
                                }
                            }
                        _ => { Outcome::OtherError(rejected.error.to_string()) }
                    };

                    (None, outcome)
                }
            TransactionResult::Abort(aborted) =>
                {
                    (None, Outcome::OtherError(aborted.reason.to_string()))
                }
        }
    }

    fn to_manifest(&self) -> TransactionManifest{

        let mut manifest = self.manifest.build();
        manifest.instructions.insert(
            0,
            transaction::model::Instruction::CallMethod {
                component_address: self.fee_payer,
                method_name: "lock_fee".to_string(),
                args: manifest_args!(dec!("100")),
            }
        );
        manifest
    }
}

pub struct MethodCallReceipt<T: ScryptoDecode>{
    fee_paid: Option<Decimal>,
    method_return: Option<T>,
    outcome: Outcome
}

impl<T: ScryptoDecode> MethodCallReceipt<T> {

    pub fn from(fee_paid: Option<Decimal>, method_return: Option<T>, outcome: Outcome) -> Self{
        Self{
            fee_paid,
            method_return,
            outcome
        }
    }

    pub fn fee_paid(&self) -> Decimal {
        self.fee_paid.clone().unwrap()
    }

    pub fn get_return(&self) -> &T {
        &self.method_return.unwrap()
    }
}