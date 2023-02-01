use lazy_static::lazy_static;
use regex::Regex;

pub enum Error {
    Success,
    AssertFailed(String),
    Other(String)
}

impl Error {

    pub fn has_been_triggered(&self, stdout: String) {

        match self {
            Error::Success =>
                {
                    lazy_static! {
                        static ref SUCCESS_RE: Regex = Regex::new("Transaction Status: COMMITTED SUCCESS").unwrap();
                    }
                    if !&SUCCESS_RE.is_match(&stdout)
                    {
                        panic!("Manifest failed!\n\
                                Transaction Output: \n\n{}", stdout);
                    }
                }
            Error::AssertFailed(expected_error) =>
                {
                    let assert_error_re: Regex = Regex::new(r#"Transaction Status: COMMITTED FAILURE: KernelError\(WasmError\(WasmError\("Trap\(Trap \{ kind: Unreachable \}\)"\)\)\)"#).unwrap();
                    let error = format!(r#"└─ \[ERROR\] Panicked at '{}'"#, expected_error);
                    let error_message_re: Regex = Regex::new(error.as_str()).unwrap();

                    if !&assert_error_re.is_match(&stdout)
                    {
                        panic!("Manifest failed with the wrong error!\n\
                                Transaction Output: \n\n{}", stdout);
                    }
                    else if !&error_message_re.is_match(&stdout)
                    {
                        panic!("Manifest panicked with the wrong error message!\n\
                                Expected Message: {}\n\
                                Got: {}\n\
                                Transaction Output: \n\n{}", expected_error, error, stdout)

                    }
                }
            Error::Other(expected_error) =>
                {
                    let other_error_re: Regex = Regex::new(format!("Transaction Status: COMMITTED FAILURE: {}", expected_error).as_str()).unwrap();

                    if !&other_error_re.is_match(&stdout)
                    {
                        panic!("Manifest failed with the wrong error!\n\
                                Expected Error: {}\n\
                                Transaction Output: \n\n{}", expected_error, stdout);
                    }
                }
        }
    }

}