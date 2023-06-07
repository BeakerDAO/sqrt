#[cfg(test)]
mod hello_world_tests {
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


        // We check that the new tokens have indeed been recognized by the TestEnvironment
        assert!(test_env.exists_resource("HelloToken") && test_env.exists_resource("test"));
    }
}

