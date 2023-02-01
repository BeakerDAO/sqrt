use crate::error::Error;
use crate::test_environment::TestEnvironment;
use crate::utils::run_manifest;

pub struct  ManifestCall<'a>{
    test_environment: &'a mut TestEnvironment,
    package_path: String,
    call_name: String,
    custom_manifest: bool,
    env_bindings: Vec<(String,String)>,
    output_manifest: bool,
    expected_error: Error
}

impl <'a> ManifestCall<'a>{

    pub fn new(test_environment: &'a mut TestEnvironment, package_path: String, call_name: &str, custom_manifest: bool) -> ManifestCall<'a> {
        ManifestCall{
            test_environment,
            package_path,
            call_name: call_name.to_string(),
            custom_manifest,
            env_bindings: vec![],
            output_manifest: false,
            expected_error: Error::Success,
        }
    }

    pub fn add_bindings(mut self, new_bindings: &mut Vec<(String, String)>) -> ManifestCall<'a> {
        self.env_bindings.append(new_bindings);
        self
    }

    pub fn add_binding(mut self, new_binding: (String, String)) -> ManifestCall<'a> {
        self.env_bindings.push(new_binding);
        self
    }

    pub fn output_manifest(mut self) -> ManifestCall<'a> {
        self.output_manifest = true;
        self
    }

    pub fn should_panic(mut self, error: Error) -> ManifestCall<'a> {
        self.expected_error = error;
        self
    }

    pub fn run(self) -> Option<String>
    {
        let (manifest_output, stdout) = run_manifest(self.package_path.as_str(), self.call_name.as_str(), self.custom_manifest, self.env_bindings);
        self.expected_error.has_been_triggered(stdout);
        self.test_environment.update();

        if self.output_manifest {
            Some(manifest_output)
        }
        else
        {
            None
        }
    }
}