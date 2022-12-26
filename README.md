# sqrt
The Scrypto Quick Rtm Testing package is a tool to easily test Scrypto packages using radix transaction manifests.

# TODO for version 1.0
- [ ] Deal with return of blueprint methods
- [ ] Allow multiple arguments return when instantiating a function
- [ ] Allow multiple possible instantiation
- [ ] Deal with blueprints state 
- [ ] Deal with returns and automatically check how things should have evolved
- [ ] Automatic implementation of method trait 

# Known Bugs

- [ ] When multiple components are instantiated at the same time, the library will fail to understand which component 
is bounded to which address.

# Usage
To use this library, for every blueprint that you want to test, create an empty struct and implement the `Blueprint` 
trait for it. This trait basically explains how the blueprint should be instantiated.
For the `gumball-machine` basic Scrypto package, this looks like:
```Rust
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


```
Then, for each blueprint, create an enum with methods and their arguments. Then implement the `Method` trait which 
explains how to call the method and its arguments.
For the `gumball-machine` basic Scrypto package, this looks like:
```Rust
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
            GumballMethods::GetPrice => { method_args![] }
            GumballMethods::BuyGumball(value) =>
                {
                    method_args![Arg::BucketArg(String::from("radix"), value.clone())]
                }
        }
    }
}
```

Then, when everything is implemented, you can start writing your tests! To test your library, you can then use the
following command:

```shell
cargo test -- --test-threads=1
```

# Examples
A wide range of examples are available in the [test](tests) folder.