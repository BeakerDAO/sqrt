#[cfg(test)]
mod hello_tests {
    use scrypto::math::Decimal;
    use sqrt::blueprint::Blueprint;
    use sqrt::method::{Arg, Method};
    use sqrt::method_args;
    use sqrt::package::Package;
    use sqrt::test_environment::TestEnvironment;

    // To define a Blueprint, we need to implement the Blueprint Trait for some object.
    // We therefore define an empty struct for which we will implement the Blueprint Trait.
    struct HelloBp {}

    impl Blueprint for HelloBp {
        // A new "Hello" blueprint is instantiated from the "instantiate_hello" method
        fn instantiation_name(&self) -> &str {
            "instantiate_hello"
        }

        // The name of the blueprint is indeed "Hello"
        fn name(&self) -> &str {
            "Hello"
        }

        // The "Hello" blueprint does not use an admin badge
        fn has_admin_badge(&self) -> bool {
            false
        }
    }

    // To test methods of a blueprint, we need to implement the Method Trait. The best way of doing so
    // is defining an enum which variants are the blueprint's methods and then implement the Method Trait
    // for it.

    // In this case, the blueprint "Hello" has only one method with no argument so we create an enum
    // with only one variant which has no arguments
    enum HelloMethods {
        FreeToken,
    }

    // We can now implement the Method trait for our enum
    impl Method for HelloMethods {
        fn name(&self) -> &str {
            match self {
                HelloMethods::FreeToken => "free_token",
            }
        }

        // We have no method that needs an argument so we always return `method_args![]` with no args
        fn args(&self) -> Option<Vec<Arg>> {
            method_args![]
        }

        // None of our methods requires the Component's admin badge to be called, so we always return false
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

        // We check that the new tokens have indeed been recognized by the TestEnvironment
        test_env.get_resource("HelloToken");
        test_env.get_resource("test");
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
        // We check that we indeed received 1 HelloToken after having called the FreeToken function
        assert_eq!(test_env.amount_owned_by_current("HelloToken"), Decimal::ONE);
    }
}
