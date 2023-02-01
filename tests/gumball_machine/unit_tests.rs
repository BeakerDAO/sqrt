#[cfg(test)]
mod gumball_tests {
    use scrypto::prelude::{dec, Decimal};
    use sqrt::blueprint::Blueprint;
    use sqrt::error::Error;
    use sqrt::method::Arg::DecimalArg;
    use sqrt::method::{Arg, Method};
    use sqrt::method_args;
    use sqrt::package::Package;
    use sqrt::test_environment::TestEnvironment;

    struct GumballBp {}

    impl Blueprint for GumballBp {
        fn instantiation_name(&self) -> &str {
            "instantiate_gumball_machine"
        }

        fn name(&self) -> &str {
            "GumballMachine"
        }

        fn has_admin_badge(&self) -> bool {
            false
        }
    }

    enum GumballMethods {
        GetPrice,
        BuyGumball(Decimal),
    }

    impl Method for GumballMethods {
        fn name(&self) -> &str {
            match self {
                GumballMethods::GetPrice => "get_price",
                GumballMethods::BuyGumball(_) => "buy_gumball",
            }
        }

        fn args(&self) -> Option<Vec<Arg>> {
            match self {
                GumballMethods::GetPrice => {
                    method_args![]
                }
                GumballMethods::BuyGumball(value) => {
                    method_args![Arg::FungibleBucketArg(String::from("radix"), value.clone())]
                }
            }
        }

        fn needs_admin_badge(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_publish() {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp {});
        let mut gumball_package = Package::new("tests/gumball_machine/package");
        gumball_package.add_blueprint("gumball", gumball_blueprint);
        test_env.publish_package("gumball", gumball_package);
    }

    #[test]
    fn test_instantiate() {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp {});
        let mut gumball_package = Package::new("tests/gumball_machine/package");
        gumball_package.add_blueprint("gumball", gumball_blueprint);
        test_env.publish_package("gumball", gumball_package);
        test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);

        test_env.get_resource("gumball");
    }

    #[test]
    fn test_get_price() {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp {});
        let mut gumball_package = Package::new("tests/gumball_machine/package");
        gumball_package.add_blueprint("gumball", gumball_blueprint);
        test_env.publish_package("gumball", gumball_package);
        test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);

        test_env.call_method(GumballMethods::GetPrice).run();
    }

    #[test]
    fn test_buy_gumball() {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp {});
        let mut gumball_package = Package::new("tests/gumball_machine/package");
        gumball_package.add_blueprint("gumball", gumball_blueprint);
        test_env.publish_package("gumball", gumball_package);
        test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);

        let xrd_owned_before_call = test_env.amount_owned_by_current("radix");
        test_env.call_method(GumballMethods::BuyGumball(dec!(15))).run();
        let new_amount_xrd_amount = test_env.amount_owned_by_current("radix");

        assert_eq!(test_env.amount_owned_by_current("gumball"), Decimal::one());
        assert!(xrd_owned_before_call - new_amount_xrd_amount > dec!("1.5"));
    }

    #[test]
    fn test_buy_gumball_not_enough() {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp {});
        let mut gumball_package = Package::new("tests/gumball_machine/package");
        gumball_package.add_blueprint("gumball", gumball_blueprint);
        test_env.publish_package("gumball", gumball_package);
        test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);
        test_env.call_method(GumballMethods::BuyGumball(dec!(1)))
            .should_panic(Error::Other("ApplicationError(BucketError(ResourceOperationError(InsufficientBalance)))".to_string()));
    }
}
