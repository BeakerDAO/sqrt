#[cfg(test)]
mod hello_tests {
    use sqrt::blueprint::Blueprint;
    use sqrt::method::{Arg, Method};
    use sqrt::method_args;
    use sqrt::package::Package;
    use sqrt::test_environment::TestEnvironment;

    struct HelloBp {}

    impl Blueprint for HelloBp {
        fn instantiate(&self, _arg_values: Vec<String>) -> (&str, Vec<String>) {
            let function_name = "instantiate_hello";
            (function_name, vec![])
        }

        fn name(&self) -> &str {
            "Hello"
        }

        fn has_admin_badge(&self) -> bool {
            false
        }
    }

    enum HelloMethods {
        FreeToken,
    }

    impl Method for HelloMethods {
        fn name(&self) -> &str {
            match self {
                HelloMethods::FreeToken => "free_token",
            }
        }

        fn args(&self) -> Option<Vec<Arg>> {
            method_args![]
        }

        fn needs_admin_badge(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_publish() {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp {});
        let mut hello_package = Package::new("tests/hello_token/package/");
        hello_package.add_blueprint("hello", hello_blueprint);
        test_env.publish_package("hello", hello_package);
    }

    #[test]
    fn test_instantiate() {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp {});
        let mut hello_package = Package::new("tests/hello_token/package/");
        hello_package.add_blueprint("hello", hello_blueprint);
        test_env.publish_package("hello", hello_package);
        test_env.new_component("hello_comp", "hello", vec![]);

        // Check that tokens have been added to list
        test_env.get_token("HelloToken");
        test_env.get_token("test");
    }

    #[test]
    fn test_free_token() {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp {});
        let mut hello_package = Package::new("tests/hello_token/package/");
        hello_package.add_blueprint("hello", hello_blueprint);
        test_env.publish_package("hello", hello_package);
        test_env.new_component("hello_comp", "hello", vec![]);

        test_env.call_method(HelloMethods::FreeToken);
    }
}
