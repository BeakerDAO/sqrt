//! Constructs a Manifest call

use crate::error::Error;
use crate::test_environment::TestEnvironment;
use crate::utils::run_manifest;

pub struct ManifestCall<'a> {
    test_environment: &'a mut TestEnvironment,
    manifest_name: Option<String>,
    custom_manifest: Option<bool>,
    env_bindings: Vec<(String, String)>,
    output_manifest: bool,
    expected_error: Error,
}

impl<'a> ManifestCall<'a> {
    /// Returns a new ManifestCall
    ///
    /// # Arguments
    /// * `test_environment` - [`TestEnvironment`] of the call
    pub fn new(test_environment: &'a mut TestEnvironment) -> ManifestCall<'a> {
        ManifestCall {
            test_environment,
            manifest_name: None,
            custom_manifest: None,
            env_bindings: vec![],
            output_manifest: false,
            expected_error: Error::Success,
        }
    }

    ///
    ///
    /// # Arguments
    /// * `test_environment` - [`TestEnvironment`] of the call
    pub fn call_manifest(mut self, manifest_name: &str, custom_manifest: bool) -> ManifestCall<'a> {
        self.manifest_name = Some(manifest_name.to_string());
        self.custom_manifest = Some(custom_manifest);
        self
    }

    /// Adds the given environment variable bindings to the [`ManifestCall`]
    ///
    /// # Arguments
    /// * `new_bindings` - bindings to add
    pub fn add_bindings(mut self, new_bindings: &mut Vec<(String, String)>) -> ManifestCall<'a> {
        self.env_bindings.append(new_bindings);
        self
    }

    /// Adds the given environment variable binding to the [`ManifestCall`]
    ///
    /// # Arguments
    /// * `new_bindings` - binding to add
    pub fn add_binding(mut self, new_binding: (String, String)) -> ManifestCall<'a> {
        self.env_bindings.push(new_binding);
        self
    }

    /// Instruction to output the manifest called
    pub fn output_manifest(mut self) -> ManifestCall<'a> {
        self.output_manifest = true;
        self
    }

    /// States that the [`ManifestCall`] should panic with the given error
    pub fn should_panic(mut self, error: Error) -> ManifestCall<'a> {
        self.expected_error = error;
        self
    }

    /// Runs a [`ManifestCall`] and returns a [`String`] if required
    pub fn run(self) -> Option<String> {
        if self.manifest_name.is_none() || self.custom_manifest.is_none() {
            panic!("Cannot run a manifest without specifying what to call")
        }

        let (manifest_output, stdout, stderr) = run_manifest(
            self.test_environment.get_current_package().path(),
            self.manifest_name.unwrap().as_str(),
            self.custom_manifest.unwrap(),
            self.env_bindings,
        );
        self.expected_error.check_error(stdout, stderr);
        self.test_environment.update();

        if self.output_manifest {
            Some(manifest_output)
        } else {
            None
        }
    }

    /// Runs a [`ManifestCall`] and returns the call output
    pub fn debug_manifest(self) -> (String, String) {
        if self.manifest_name.is_none() || self.custom_manifest.is_none() {
            panic!("Cannot debug a manifest without specifying what to call")
        }

        let (_, stdout, stderr) = run_manifest(
            self.test_environment.get_current_package().path(),
            self.manifest_name.unwrap().as_str(),
            self.custom_manifest.unwrap(),
            self.env_bindings,
        );
        self.test_environment.update();

        (stdout, stderr)
    }
}
