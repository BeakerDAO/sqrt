#[cfg(test)]
mod rns_tests {
    use scrypto::dec;
    use scrypto::math::Decimal;
    use sqrt::blueprint::Blueprint;
    use sqrt::method::Arg::{BucketArg, ComponentAddressArg, ProofArg, StringArg, U8};
    use sqrt::method::{Arg, Method};
    use sqrt::method_args;
    use sqrt::package::Package;
    use sqrt::test_environment::TestEnvironment;

    struct RNSBp {}

    impl Blueprint for RNSBp {
        fn instantiate(&self, _arg_values: Vec<String>) -> (&str, Vec<String>) {
            let function_name = "instantiate_rns";
            let args = vec![
                String::from("1"),
                String::from("0.01"),
                String::from("0.01"),
            ];

            (function_name, args)
        }

        fn name(&self) -> &str {
            "RadixNameService"
        }

        fn has_admin_badge(&self) -> bool {
            true
        }
    }

    enum RNSMethods {
        RegisterName(String, String, u8, Decimal),
        UnregisterName,
        UpdateAddress(String, Decimal),
        WithdrawFees,
    }

    impl Method for RNSMethods {
        fn name(&self) -> &str {
            match self {
                RNSMethods::RegisterName(_, _, _, _) => "register_name",
                RNSMethods::UnregisterName => "unregister_name",
                RNSMethods::UpdateAddress(_, _) => "update_address",
                RNSMethods::WithdrawFees => "withdraw_fees",
            }
        }

        fn args(&self) -> Option<Vec<Arg>> {
            match self {
                RNSMethods::RegisterName(name, target_address, reserve_years, deposit_amount) => {
                    method_args![
                        StringArg(name.clone()),
                        ComponentAddressArg(target_address.clone()),
                        U8(*reserve_years),
                        BucketArg(String::from("radix"), *deposit_amount)
                    ]
                }
                RNSMethods::UnregisterName => {
                    method_args![BucketArg(String::from("DomainName"), Decimal::one())]
                }
                RNSMethods::UpdateAddress(new_address, fee) => {
                    method_args![
                        ProofArg(String::from("DomainName")),
                        ComponentAddressArg(new_address.clone()),
                        BucketArg(String::from("radix"), *fee)
                    ]
                }
                RNSMethods::WithdrawFees => {
                    method_args![]
                }
            }
        }

        fn needs_admin_badge(&self) -> bool {
            match self {
                RNSMethods::WithdrawFees => true,
                _ => false,
            }
        }
    }

    #[test]
    fn test_publish() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
    }

    #[test]
    fn test_instantiate() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
        test_env.new_component("rns_comp", "rns", "rns", vec![]);

        test_env.get_token("DomainName").unwrap();
    }

    #[test]
    fn test_register_name() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
        test_env.new_component("rns_comp", "rns", "rns", vec![]);

        let current_account = test_env.get_current_account();
        test_env.call_method(
            "rns_comp",
            RNSMethods::RegisterName(
                String::from("test.xrd"),
                String::from(current_account.address()),
                1,
                dec!("15"),
            ),
        );
        let owned_nft = test_env.amount_owned_by_current("DomainName");
        assert_eq!(owned_nft, Decimal::one());
    }

    #[test]
    fn test_unregister() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
        test_env.new_component("rns_comp", "rns", "rns", vec![]);

        let current_account = test_env.get_current_account();
        test_env.call_method(
            "rns_comp",
            RNSMethods::RegisterName(
                String::from("test.xrd"),
                String::from(current_account.address()),
                1,
                dec!("15"),
            ),
        );
        let owned_nft = test_env.amount_owned_by_current("DomainName");
        assert_eq!(owned_nft, Decimal::one());

        test_env.call_method("rns_comp", RNSMethods::UnregisterName);
        let owned_nft = test_env.amount_owned_by_current("DomainName");
        assert_eq!(owned_nft, Decimal::zero());
    }

    #[test]
    fn test_update_address() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
        test_env.new_component("rns_comp", "rns", "rns", vec![]);

        let current_account = test_env.get_current_account();
        test_env.call_method(
            "rns_comp",
            RNSMethods::RegisterName(
                String::from("test.xrd"),
                String::from(current_account.address()),
                1,
                dec!("15"),
            ),
        );
        let owned_nft = test_env.amount_owned_by_current("DomainName");
        assert_eq!(owned_nft, Decimal::one());

        test_env.create_account("test");
        let account = test_env.get_account("test").unwrap();
        test_env.call_method(
            "rns_comp",
            RNSMethods::UpdateAddress(String::from(account.address()), dec!("15")),
        );
    }

    #[test]
    fn test_withdraw_fees() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
        test_env.new_component("rns_comp", "rns", "rns", vec![]);

        let current_account = test_env.get_current_account();
        test_env.call_method(
            "rns_comp",
            RNSMethods::RegisterName(
                String::from("test.xrd"),
                String::from(current_account.address()),
                1,
                dec!("15"),
            ),
        );
        let owned_nft = test_env.amount_owned_by_current("DomainName");
        assert_eq!(owned_nft, Decimal::one());

        test_env.call_method("rns_comp", RNSMethods::WithdrawFees);
    }

    #[test]
    #[should_panic]
    fn test_withdraw_fees_fail() {
        let mut test_env = TestEnvironment::new();
        let rns_blueprint = Box::new(RNSBp {});
        let mut rns_package = Package::new("tests/assets/radix-name-service");
        rns_package.add_blueprint("rns", rns_blueprint);
        test_env.publish_package("rns", rns_package);
        test_env.new_component("rns_comp", "rns", "rns", vec![]);

        let current_account = test_env.get_current_account();
        test_env.call_method(
            "rns_comp",
            RNSMethods::RegisterName(
                String::from("test.xrd"),
                String::from(current_account.address()),
                1,
                dec!("15"),
            ),
        );
        let owned_nft = test_env.amount_owned_by_current("DomainName");
        assert_eq!(owned_nft, Decimal::one());

        test_env.create_account("test");
        test_env.set_current_account("test");

        test_env.call_method("rns_comp", RNSMethods::WithdrawFees);
    }

    #[test]
    fn test() {
        let mut test_env = TestEnvironment::new();
        test_env.create_fixed_supply_token("usd", dec!(1000));
    }
}
