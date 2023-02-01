#[cfg(test)]
mod radiswap_test {
    use scrypto::prelude::{dec, Decimal};
    use sqrt::blueprint::Blueprint;
    use sqrt::method::Arg::{DecimalArg, FungibleBucketArg, StringArg};
    use sqrt::method::{Arg, Method};
    use sqrt::method_args;
    use sqrt::package::Package;
    use sqrt::test_environment::TestEnvironment;

    struct RadiSwapBp {}

    impl Blueprint for RadiSwapBp {
        fn instantiation_name(&self) -> &str {
            "instantiate_pool"
        }

        fn name(&self) -> &str {
            "Radiswap"
        }

        fn has_admin_badge(&self) -> bool {
            false
        }
    }

    enum RadiSwapMethods {
        AddLiquidity(String, Decimal, String, Decimal),
        RemoveLiquidity(String, Decimal),
        Swap(String, Decimal),
    }

    impl Method for RadiSwapMethods {
        fn name(&self) -> &str {
            match self {
                RadiSwapMethods::AddLiquidity(_, _, _, _) => "add_liquidity",
                RadiSwapMethods::RemoveLiquidity(_, _) => "remove_liquidity",
                RadiSwapMethods::Swap(_, _) => "swap",
            }
        }

        fn args(&self) -> Option<Vec<Arg>> {
            match self {
                RadiSwapMethods::AddLiquidity(a_name, a_amount, b_name, b_amount) => {
                    method_args![
                        FungibleBucketArg(a_name.clone(), a_amount.clone()),
                        FungibleBucketArg(b_name.clone(), b_amount.clone())
                    ]
                }
                RadiSwapMethods::RemoveLiquidity(lp_token_name, amount) => {
                    method_args![FungibleBucketArg(lp_token_name.clone(), amount.clone())]
                }
                RadiSwapMethods::Swap(input_tokens_name, amount) => {
                    method_args![FungibleBucketArg(input_tokens_name.clone(), amount.clone())]
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
        let radiswap_blueprint = Box::new(RadiSwapBp {});
        let mut radiswap_package = Package::new("tests/radiswap/package/");
        radiswap_package.add_blueprint("radiswap_bp", radiswap_blueprint);
        test_env.publish_package("radiswap_pkg", radiswap_package);
    }

    #[test]
    fn test_instantiate() {
        let mut test_env = TestEnvironment::new();
        let radiswap_blueprint = Box::new(RadiSwapBp {});
        let mut radiswap_package = Package::new("tests/radiswap/package/");
        radiswap_package.add_blueprint("radiswap_bp", radiswap_blueprint);
        test_env.publish_package("radiswap_pkg", radiswap_package);

        test_env.create_fixed_supply_token("usd", dec!(100000));
        test_env.create_fixed_supply_token("btc", dec!(100000));
        let args = vec![
            FungibleBucketArg("usd".to_string(), dec!(1000)),
            FungibleBucketArg("btc".to_string(), dec!(100)),
            DecimalArg(dec!(1)),
            StringArg("LP".to_string()),
            StringArg("USD-BTC LP".to_string()),
            StringArg("".to_string()),
            DecimalArg(dec!("0.003")),
        ];
        test_env.new_component("lp_comp", "radiswap_bp", args);

        let lp_tokens_owned = test_env.amount_owned_by_current("usd-btc lp");
        let usd_owned = test_env.amount_owned_by_current("usd");
        let btc_owned = test_env.amount_owned_by_current("btc");

        assert_eq!(lp_tokens_owned, dec!(1));
        assert_eq!(usd_owned, dec!(99000));
        assert_eq!(btc_owned, dec!(99900));
    }

    #[test]
    fn test_add_liquidity() {
        let mut test_env = TestEnvironment::new();
        let radiswap_blueprint = Box::new(RadiSwapBp {});
        let mut radiswap_package = Package::new("tests/radiswap/package/");
        radiswap_package.add_blueprint("radiswap_bp", radiswap_blueprint);
        test_env.publish_package("radiswap_pkg", radiswap_package);

        test_env.create_fixed_supply_token("usd", dec!(100000));
        test_env.create_fixed_supply_token("btc", dec!(100000));
        let args = vec![
            FungibleBucketArg("usd".to_string(), dec!(1000)),
            FungibleBucketArg("btc".to_string(), dec!(100)),
            DecimalArg(dec!(1)),
            StringArg("LP".to_string()),
            StringArg("USD-BTC LP".to_string()),
            StringArg("".to_string()),
            DecimalArg(dec!("0.003")),
        ];
        test_env.new_component("lp_comp", "radiswap_bp", args);

        test_env.call_method(RadiSwapMethods::AddLiquidity(
            "usd".to_string(),
            dec!(1000),
            "btc".to_string(),
            dec!(100),
        )).run();

        let lp_tokens_owned = test_env.amount_owned_by_current("usd-btc lp");
        let usd_owned = test_env.amount_owned_by_current("usd");
        let btc_owned = test_env.amount_owned_by_current("btc");

        assert_eq!(lp_tokens_owned, dec!(2));
        assert_eq!(usd_owned, dec!(98000));
        assert_eq!(btc_owned, dec!(99800));
    }

    #[test]
    fn test_remove_liquidity() {
        let mut test_env = TestEnvironment::new();
        let radiswap_blueprint = Box::new(RadiSwapBp {});
        let mut radiswap_package = Package::new("tests/radiswap/package/");
        radiswap_package.add_blueprint("radiswap_bp", radiswap_blueprint);
        test_env.publish_package("radiswap_pkg", radiswap_package);

        test_env.create_fixed_supply_token("usd", dec!(100000));
        test_env.create_fixed_supply_token("btc", dec!(100000));
        let args = vec![
            FungibleBucketArg("usd".to_string(), dec!(1000)),
            FungibleBucketArg("btc".to_string(), dec!(100)),
            DecimalArg(dec!(1)),
            StringArg("LP".to_string()),
            StringArg("USD-BTC LP".to_string()),
            StringArg("".to_string()),
            DecimalArg(dec!("0.003")),
        ];
        test_env.new_component("lp_comp", "radiswap_bp", args);

        test_env.call_method(RadiSwapMethods::RemoveLiquidity(
            "usd-btc lp".to_string(),
            dec!(1),
        )).run();

        let lp_tokens_owned = test_env.amount_owned_by_current("usd-btc lp");
        let usd_owned = test_env.amount_owned_by_current("usd");
        let btc_owned = test_env.amount_owned_by_current("btc");

        assert_eq!(lp_tokens_owned, dec!(0));
        assert_eq!(usd_owned, dec!(100000));
        assert_eq!(btc_owned, dec!(100000));
    }

    #[test]
    fn test_swap() {
        let mut test_env = TestEnvironment::new();
        let radiswap_blueprint = Box::new(RadiSwapBp {});
        let mut radiswap_package = Package::new("tests/radiswap/package/");
        radiswap_package.add_blueprint("radiswap_bp", radiswap_blueprint);
        test_env.publish_package("radiswap_pkg", radiswap_package);

        test_env.create_fixed_supply_token("usd", dec!(100000));
        test_env.create_fixed_supply_token("btc", dec!(100000));
        let args = vec![
            FungibleBucketArg("usd".to_string(), dec!(1000)),
            FungibleBucketArg("btc".to_string(), dec!(100)),
            DecimalArg(dec!(1)),
            StringArg("LP".to_string()),
            StringArg("USD-BTC LP".to_string()),
            StringArg("".to_string()),
            DecimalArg(dec!("0.003")),
        ];
        test_env.new_component("lp_comp", "radiswap_bp", args);

        test_env.call_method(RadiSwapMethods::Swap("usd".to_string(), dec!(1000))).run();

        let usd_owned = test_env.amount_owned_by_current("usd");
        let btc_owned = test_env.amount_owned_by_current("btc");

        assert_eq!(usd_owned, dec!(98000));
        assert_eq!(btc_owned, dec!("99949.924887330996494743"));
    }
}
