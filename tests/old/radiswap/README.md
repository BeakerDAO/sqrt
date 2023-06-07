# RadiSwap

We explain here how `SQRT` can be used to test the RadiSwap repository.
To test the blueprint, we have to implement the `Blueprint` and the `Method` traits.

## Blueprint trait implementation

The purpose of the `Blueprint` trait is to explain to `SQRT` how to instantiate a new component.
In our case, one has to call the function:

```Rust
pub fn instantiate_pool(
    a_tokens: Bucket,
    b_tokens: Bucket,
    lp_initial_supply: Decimal,
    lp_symbol: String,
    lp_name: String,
    lp_url: String,
    fee: Decimal,
) -> (ComponentAddress, Bucket) {}
```

Therefore, we make the following implementation for the trait:
```Rust
// To define a Blueprint, we need to implement the Blueprint Trait for some object.
// We therefore define an empty struct for which we will implement the Blueprint Trait.
struct RadiSwapBp {}

impl Blueprint for RadiSwapBp {
    fn instantiation_name(&self) -> &str {
        "instantiate_pool"
    }

    fn name(&self) -> &str {
        "Radiswap"
    }

    fn has_admin_badge(&self) -> AdminBadge {
        AdminBadge::None
    }
}
```

We can now instantiate a new blueprint in our tests in the following way:
```Rust
#[test]
fn test_instantiate() {
    // We create a new TestEnvironment. When doing so, a default account is created and is referenced by "default"
    let mut test_env = TestEnvironment::new();
    
    // Create a new instance of RadiSwapBp for which we have implemented the Blueprint trait
    let radiswap_blueprint = Box::new(RadiSwapBp {});
    
    // We create a new virtual package for the Scrypto package RadiSwap
    let mut radiswap_package = Package::new("tests/radiswap/package/");
    // We add the blueprint "Radiswap" to the package and we give it the name "radiswap_bp" so that we can find it later
    radiswap_package.add_blueprint("radiswap_bp", radiswap_blueprint);
    // We can now publish the new package and we give it the name "radiswap_pkg" so that we can find it easily later
    // This package will be used as the default package
    test_env.publish_package("radiswap_pkg", radiswap_package);
    
    // To instantiate a new LP, we need two tokens
    test_env.create_fixed_supply_token("usd", dec!(100000));
    test_env.create_fixed_supply_token("btc", dec!(100000));
    
    // We can now instantiate a new component (which will be referenced as "lp_comp")
    // We first create a vector with the desired arguments for the instantiation of the blueprint.
    let args = vec![
        FungibleBucketArg("usd".to_string(), dec!(1000)),
        FungibleBucketArg("btc".to_string(), dec!(100)),
        DecimalArg(dec!(1)),
        StringArg("LP".to_string()),
        StringArg("USD-BTC LP".to_string()),
        StringArg("".to_string()),
        DecimalArg(dec!("0.003"))
    ];
    test_env.new_component("lp_comp", "radiswap_bp", args);
    
    // We check that the current account has the right amount of tokens
    let lp_tokens_owned = test_env.amount_owned_by_current("usd-btc lp");
    let usd_owned = test_env.amount_owned_by_current("usd");
    let btc_owned = test_env.amount_owned_by_current("btc");

    assert_eq!(lp_tokens_owned, dec!(1));
    assert_eq!(usd_owned, dec!(99000));
    assert_eq!(btc_owned, dec!(99900));
}
```

## Method trait implementation

Now that we can instantiate a `Radiswap` component, we would like to test its methods.
To be able to test the methods of a blueprint, we need to explain to `SQRT` how we want to call it.  
The standard way of doing so is to create an `Enum` with one variant for every method of the blueprint. The arguments of
the variant will be used to call the method with specific arguments.  
In our case, a `Radiswap` component has 4 public methods:
```Rust
impl RadixNameService {
    pub fn add_liquidity(
        &mut self,
        mut a_tokens: Bucket,
        mut b_tokens: Bucket,
    ) -> (Bucket, Bucket) {}
    pub fn remove_liquidity(&mut self, lp_tokens: Bucket) -> (Bucket, Bucket) {}
    pub fn swap(&mut self, input_tokens: Bucket) -> Bucket {}
    pub fn get_pair(&self) -> (ResourceAddress, ResourceAddress) {}
}
```
Because SQRT does not support checking the return of a method yet, we cannot properly test `get_pair`.
We therefore create an `Enum` with 3 variants:
enum RadiSwapMethods {
AddLiquidity(String, Decimal, String, Decimal),
RemoveLiquidity(String, Decimal),
Swap(String, Decimal),
}
```Rust
// There is no restriction on the name of the variants but it is recommended to use a name close to the name of the
// method it is referring to
enum RadiSwapMethods {
    AddLiquidity(
        // Name of the first token
        String, 
        // Amount of the first token to send
        Decimal,
        // Name of the second token
        String,
        // Amount of the second token to send
        Decimal),
    
    RemoveLiquidity(
        // Name of the LP token
        String,
        // Amount of LP tokens to send
        Decimal),
    
    Swap(
        // Name of the token to trade
        String,
        // Amount of token to trade
        Decimal),
}
```
The choice of arguments for the variants is up to the user of the library. The implementation of the `Method` trait will
explain how to make the call.  
Here, we made certain decisions but other ones could have worked too.

Implementing the trait `Method` depends on the choice of the variants of the `Enum`. For our choice, the following code
implements the trait:
```Rust
impl Method for RadiSwapMethods {
    
    // We explain here what is the name of the method associated to a variant
    fn name(&self) -> &str {
        match self {
            RadiSwapMethods::AddLiquidity(_, _, _, _) => { "add_liquidity" }
            RadiSwapMethods::RemoveLiquidity(_, _) => { "remove_liquidity" }
            RadiSwapMethods::Swap(_, _) => { "swap" }
        }
    }

    fn args(&self) -> Option<Vec<Arg>> {
        match self {
            RadiSwapMethods::AddLiquidity(a_name, a_amount, b_name, b_amount) =>
                {
                    method_args![
                            FungibleBucketArg(a_name.clone(), a_amount.clone()),
                            FungibleBucketArg(b_name.clone(), b_amount.clone())
                        ]
                }
            RadiSwapMethods::RemoveLiquidity(lp_token_name, amount) =>
                {
                    method_args![
                            FungibleBucketArg(lp_token_name.clone(), amount.clone())
                        ]
                }
            RadiSwapMethods::Swap(input_tokens_name, amount) =>
                {
                    method_args![
                            FungibleBucketArg(input_tokens_name.clone(), amount.clone())
                        ]
                }
        }
    }

    fn needs_admin_badge(&self) -> bool {
        false
    }

    fn custom_manifest_name(&self) -> Option<&str> {
        None
    }
}
```

## Testing the methods

We can now test all methods. Note that a call to a method will create a generic Transaction Manifest that can be found
in the directory `package/rtm/`. As the test are pretty straightforward, we don't comment them here.

