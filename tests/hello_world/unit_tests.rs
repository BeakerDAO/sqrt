#[cfg(test)]
mod hello_world_tests {
    use radix_engine::types::Decimal;
    use radix_engine_interface::manifest_args;
    use radix_engine_interface::sbor::Encoder;
    use sqrt::test_environment::TestEnvironment;

    #[test]
    fn test_publish() {
        let mut test_env = TestEnvironment::new();
        test_env.new_package("hello_world", "tests/hello_world/package/");
    }

    #[test]
    fn test_instantiate() {
        let mut test_env = TestEnvironment::new();
        test_env.new_package("hello_world", "tests/hello_world/package/");
        test_env.new_component("hello_comp", "hello_world", "Hello", "instantiate_hello", manifest_args!());

        // We check that the new tokens have indeed been recognized by the test environment.
        assert!(test_env.exists_resource("HelloToken") && test_env.exists_resource("test"));
    }

    #[test]
    fn test_free_token() {
        let mut test_env = TestEnvironment::new();
        test_env.new_package("hello_world", "tests/hello_world/package/");
        test_env.new_component("hello_comp", "hello_world", "Hello", "instantiate_hello", manifest_args!());

        test_env.call_method(HelloMethods::FreeToken).run();
        // We check that we indeed received 1 HelloToken after having called the FreeToken function
        assert_eq!(test_env.amount_owned_by_current("HelloToken"), Decimal::ONE);
    }
}

