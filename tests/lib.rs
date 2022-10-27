#[cfg(test)]
mod hello_tests
{
    use suft::blueprint::Blueprint;
    use suft::method::{Arg, Method};
    use suft::method::Arg::Other;
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
            None
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

    #[test]
    fn test_free_token()
    {
        let mut test_env = TestEnvironment::new();
        let hello_blueprint = Box::new(HelloBp{});
        let hello_package = Package::from(vec![(String::from("hello"),hello_blueprint)]);
        test_env.publish_package("hello", hello_package, "tests/assets/hello-token/");
        test_env.new_component("hello_comp", "hello", "hello");

        test_env.call_method("hello_comp", HelloMethods::FreeToken);

    }
}


#[cfg(test)]
mod gumball_tests
{
    use scrypto::dec;
    use scrypto::prelude::{Decimal, RADIX_TOKEN};
    use suft::blueprint::Blueprint;
    use suft::method::{Arg, Method};
    use suft::method::Arg::Other;
    use suft::test_environment::TestEnvironment;
    use suft::package::Package;

    struct GumballBp {}

    impl Blueprint for GumballBp
    {
        fn instantiate(&self) -> (&str, Vec<&str>) {
            let name = "instantiate_gumball_machine";
            let args = vec!["1.5"];

            (name, args)
        }

        fn name(&self) -> &str {
            "GumballMachine"
        }
    }

    enum GumballMethods
    {
        GetPrice,
        BuyGumball(Decimal)
    }

    impl Method for GumballMethods
    {
        fn name(&self) -> &str {
            match self
            {
                GumballMethods::GetPrice => { "get_price" }
                GumballMethods::BuyGumball(_) => { "buy_gumball" }
            }
        }

        fn args(&self) -> Option<Vec<Arg>> {
            match self
            {
                GumballMethods::GetPrice => { Some(vec![Other(String::from(""))]) }
                GumballMethods::BuyGumball(value) =>
                    {
                        Some(vec![Arg::Bucket(String::from("radix"), value.clone())])
                    }
            }
        }
    }


    #[test]
    fn test_publish()
    {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp{});
        let gumball_package = Package::from(vec![(String::from("gumball"),gumball_blueprint)]);
        test_env.publish_package("gumball", gumball_package, "tests/assets/gumball-machine/");
    }

    #[test]
    fn test_instantiate()
    {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp{});
        let gumball_package = Package::from(vec![(String::from("gumball"),gumball_blueprint)]);
        test_env.publish_package("gumball", gumball_package, "tests/assets/gumball-machine/");
        test_env.new_component("gumball_comp", "gumball", "gumball");
    }

    #[test]
    fn test_get_price()
    {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp{});
        let gumball_package = Package::from(vec![(String::from("gumball"),gumball_blueprint)]);
        test_env.publish_package("gumball", gumball_package, "tests/assets/gumball-machine/");
        test_env.new_component("gumball_comp", "gumball", "gumball");

        test_env.call_method("gumball_comp", GumballMethods::GetPrice);
    }

    #[test]
    fn test_buy_gumball()
    {
        let mut test_env = TestEnvironment::new();
        let gumball_blueprint = Box::new(GumballBp{});
        let gumball_package = Package::from(vec![(String::from("gumball"),gumball_blueprint)]);
        test_env.publish_package("gumball", gumball_package, "tests/assets/gumball-machine/");
        test_env.new_component("gumball_comp", "gumball", "gumball");

        test_env.call_method("gumball_comp", GumballMethods::BuyGumball(dec!(15)));
    }
}