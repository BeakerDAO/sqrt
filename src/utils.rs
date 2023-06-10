use radix_engine::errors::{KernelError, RejectionError, RuntimeError};
use radix_engine::transaction::{TransactionReceipt, TransactionResult};
use radix_engine::types::Decimal;
use radix_engine::wasm::WasmRuntimeError;
use crate::calls::Outcome;

pub fn get_outcome_and_fees(receipt: &TransactionReceipt) -> (Option<Decimal>, Outcome)
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
                                                        _ => Outcome::OtherError(wasm_error.to_string())
                                                    }
                                                }
                                            _ => Outcome::OtherError(String::new())
                                        }
                                    }
                                _ => Outcome::OtherError(error.to_string())
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