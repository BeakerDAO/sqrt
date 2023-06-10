#[cfg(test)]
mod hello_world_tests {
    use sqrt::blueprint::{AdminBadge, Blueprint};
    use sqrt::call_args;
    use sqrt::test_environment::TestEnvironment;

    struct HelloWorldBp {}

    impl Blueprint for HelloWorldBp {
        fn instantiation_name(&self) -> &str {
            "instantiate_hello"
        }

        fn name(&self) -> &str {
            "Hello"
        }

        fn admin_badge_type(&self) -> AdminBadge {
            AdminBadge::None
        }
    }

    #[test]
    fn test_publish() {
        let mut test_env = TestEnvironment::new();
        test_env.new_package("hello_world", "tests/hello_world/package/");
    }

    #[test]
    fn test_instantiate() {

        let mut test_env = TestEnvironment::new();
        test_env.new_package("hello_world", "tests/hello_world/package/");
        let hello_bp = HelloWorldBp{};
        test_env.new_component("hello_comp", hello_bp, call_args!());

        // We check that the new tokens have indeed been recognized by the test environment.
        assert!(test_env.exists_resource("HelloToken") && test_env.exists_resource("test"));
    }
}

