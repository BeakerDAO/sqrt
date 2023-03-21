//! Handles errors from calls to contract

use crate::error::Error::{AssertFailed, Other};
use lazy_static::lazy_static;
use regex::Regex;

pub enum Error {
    /// States that no error is expected
    Success,
    /// States that an assertion is expected to fail with a given message
    AssertFailed(String),
    /// States that another error should happen
    Other(String),
}

impl Error {
    /// Checks that the error has happened
    ///
    /// # Arguments
    /// * `stdout` - String containing the stdout to check
    pub fn check_error(&self, stdout: String, stderr: String) {


        if stdout == String::from("")
        {
            panic!("There was an error when trying to run manifest:\n{}", stderr);
        }

        match self {
            Error::Success => {
                lazy_static! {
                    static ref SUCCESS_RE: Regex =
                        Regex::new("Transaction Status: COMMITTED SUCCESS").unwrap();
                }
                if !&SUCCESS_RE.is_match(&stdout) {
                    panic!(
                        "Manifest failed!\n\
                                Transaction Output: \n\n{}",
                        stdout
                    );
                }
            }
            AssertFailed(expected_error) => {
                let assert_error_re: Regex = Regex::new(r#"Transaction Status: COMMITTED FAILURE: KernelError\(WasmRuntimeError\(InterpreterError\("Trap\(Trap \{ kind: Unreachable \}\)"\)\)\)"#).unwrap();
                let error = format!(r#"└─ \[ERROR\] Panicked at '{}'"#, *expected_error);
                let error_message_re: Regex = Regex::new(error.as_str()).unwrap();

                if !assert_error_re.is_match(&stdout) {
                    panic!(
                        "Manifest failed with the wrong error!\n\
                                Transaction Output: \n\n{}",
                        stdout
                    );
                } else if !error_message_re.is_match(&stdout) {
                    panic!(
                        "Manifest panicked with the wrong error message!\n\
                                Expected Message: {}\n\
                                Transaction Output: \n\n{}",
                        expected_error, stdout
                    )
                }
            }
            Other(expected_error) => {
                let other_error_re: Regex = Regex::new(
                    format!("Transaction Status: COMMITTED FAILURE: {}", expected_error).as_str(),
                )
                .unwrap();

                if !&other_error_re.is_match(&stdout) {
                    panic!(
                        "Manifest failed with the wrong error!\n\
                                Expected Error: {}\n\
                                Transaction Output: \n\n{}",
                        expected_error, stdout
                    );
                }
            }
        }
    }

    fn to_regex_str(str: &str) -> String {
        str.replace("(", r"\(")
            .replace(")", r"\)")
            .replace("{", r"\{")
            .replace("}", r"\}")
            .replace("[", r"\[")
            .replace("]", r"\]")
    }
}

pub fn assert_fail(error_message: &str) -> Error {
    AssertFailed(Error::to_regex_str(error_message))
}

pub fn other_error(error: &str) -> Error {
    Other(Error::to_regex_str(error))
}
