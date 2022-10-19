#[cfg(test)]
mod tests
{
    use scrypto::resource::Vault;
    use suft::blueprint::Blueprint;
    use suft::test_environment::TestEnvironment;
    use suft::package::Package;
    
    struct HelloBp
    {}

    impl Blueprint for HelloBp
    {
        fn instantiate(&self) -> (String, Vec<String>)
        {
            let function_name = String::from("instantiate_hello");
            (function_name, vec![])
        }

        fn name(&self) -> &str
        {
            "Hello"
        }
    }
    
    #[test]
    fn test_publish()
    {
        let mut test_env = TestEnvironment::new();

        let hello_blueprint = Box::new(HelloBp{});
        let hello_package = Package::from(vec![(String::from("hello"),hello_blueprint)]);
        test_env.publish_package("hello", hello_package, "tests/assets/hello-token/");
    }

    #[test]
    fn test_instantiate()
    {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp{});
        let hello_package = Package::from(vec![(String::from("hello"),hello_blueprint)]);
        test_env.publish_package("hello", hello_package, "tests/assets/hello-token/");
        test_env.new_component("hello_comp", "hello", "hello");

    }
}
