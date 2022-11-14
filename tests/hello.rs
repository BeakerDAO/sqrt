#[cfg(test)]
mod hello_tests
{
    use suft::blueprint::Blueprint;
    use suft::method::{Arg, Method};
    use suft::method_args;
    use suft::test_environment::TestEnvironment;
    use suft::package::Package;


    struct HelloBp {}

    impl Blueprint for HelloBp
    {
        fn instantiate(&self) -> (&str, Vec<&str>)
        {
            let function_name = "instantiate_hello";
            (function_name, vec![])
        }

        fn name(&self) -> &str
        {
            "Hello"
        }
    }

    enum HelloMethods
    {
        FreeToken
    }

    impl Method for HelloMethods
    {
        fn name(&self) -> &str {
            match self
            {
                HelloMethods::FreeToken => { "free_token" }
            }
        }

        fn args(&self) -> Option<Vec<Arg>>
        {
            method_args![]
        }
    }

    #[test]
    fn test_publish()
    {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp{});
        let mut hello_package = Package::new("tests/assets/hello-token/");
        hello_package.add_blueprint("hello", hello_blueprint);
        test_env.publish_package("hello", hello_package);
    }

    #[test]
    fn test_instantiate()
    {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp{});
        let mut hello_package = Package::new("tests/assets/hello-token/");
        hello_package.add_blueprint("hello", hello_blueprint);
        test_env.publish_package("hello", hello_package);
        test_env.new_component("hello_comp", "hello", "hello");

        // Check that tokens have been added to list
        test_env.get_token("HelloToken").unwrap();
        test_env.get_token("test").unwrap();
    }

    #[test]
    fn test_free_token()
    {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp{});
        let mut hello_package = Package::new("tests/assets/hello-token/");
        hello_package.add_blueprint("hello", hello_blueprint);
        test_env.publish_package("hello", hello_package);
        test_env.new_component("hello_comp", "hello", "hello");

        test_env.call_method("hello_comp", HelloMethods::FreeToken);

    }
}

