# Gumball Machine

We explain here how `SQRT` can be used to test the Gumball Machine repository.
To test the blueprint, we have to implement the `Blueprint` and the `Method` traits.

## Blueprint trait implementation

The purpose of the `Blueprint` trait is to explain to `SQRT` how to instantiate a new component.
In our case, one has to call the function:

```Rust
pub fn instantiate_gumball_machine(price: Decimal)-> ComponentAddress {}
```
Therefore, we make the following implementation for the trait:

```Rust
// To define a Blueprint, we need to implement the Blueprint Trait for some object.
// We therefore define an empty struct for which we will implement the Blueprint Trait.
struct GumballBp {}

impl Blueprint for GumballBp {

    fn instantiation_name(&self) -> &str {
        "instantiate_gumball_machine"
    }

    fn name(&self) -> &str {
        "GumballMachine"
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
    
    // Create a new instance of GumballBp for which we have implemented the Blueprint trait
    let gumball_blueprint = Box::new(GumballBp {});
    
    // We create a new virtual package for the Scrypto package Gumball
    let mut gumball_package = Package::new("tests/gumball_machine/package");
    // We add the blueprint "Gumball" to the package and we give it the name "gumball_bp" so that we can find it later
    gumball_package.add_blueprint("gumball_bp", gumball_blueprint);
    // We can now publish the new package and we give it the name "gumball_pkg" so that we can find it easily later
    // This package will be used as the default package
    test_env.publish_package("gumball", gumball_package);
    
    // We can now instantiate a new component (which will be referenced as "gumball_comp")
    // We give the argument DecimalArg(dec!("1.5")) because the component needs a Decimal price for the gumballs
    test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);

    // When instantiating a new Gumball component, the blueprint creates a new token called "gumball"
    // We check that the new token has indeed been recognized by the TestEnvironment
    test_env.get_resource("gumball");
}
```

## Method trait implementation

Now that we can instantiate a `Gumball` component, we would like to test its methods.
To be able to test the methods of a blueprint, we need to explain to `SQRT` how we want to call it.  
The standard way of doing so is to create an `Enum` with one variant for every method of the blueprint. The arguments of 
the variant will be used to call the method with specific arguments.  
In our case, a `Gumball` component has two methods:
```Rust
 impl GumballMachine {
    pub fn get_price(&self) -> Decimal {}
    pub fn buy_gumball(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {}
}
```

We therefore create an `Enum` with two variants:
```Rust
// There is no restriction on the name of the variants but it is recommended to use a name close to the name of the
// method it is referring to
enum GumballMethods {
    GetPrice,
    BuyGumball(Decimal),
}
```
The choice of arguments for the variants is up to the user of the library. The implementation of the `Method` trait will
explain how to make the call. Here, we decided that during test we want to call `buy_gumball` with an amount of XRD to 
send, which explains the `Decimal` argument for the variant `BuyGumball`. 

Implementing the trait `Method` depends on the choice of the variants of the `Enum`. For our choice, the following code 
implements the trait:
```Rust
impl Method for GumballMethods {
    
    // We explain here what is the name of the method associated to a variant
    fn name(&self) -> &str {
        match self {
            GumballMethods::GetPrice => "get_price",
            GumballMethods::BuyGumball(_) => "buy_gumball",
        }
    }
    
    // We explain here how to transform the arguments of the variants into proper arguments
    fn args(&self) -> Option<Vec<Arg>> {
        match self {
            
            // For the method get_price, there is nothing to do because the method does not take any argument
            GumballMethods::GetPrice => {
                method_args![]
            }
            
            // For the method buy_gumball, it takes as input a Bucket containing XRD tokens
            // Therefore, we tell SQRT that the first argument is a Bucket containing Fungible tokens with the name 
            // "radix" and that the bucket should contain a total of "value" of them.
            // Note that token names is not case sensitive, therefore RaDix would have worked too.
            GumballMethods::BuyGumball(value) => {
                method_args![Arg::FungibleBucketArg(String::from("radix"), value.clone())]
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

We can now test both methods. Note that a call to a method will create a generic Transaction Manifest that can be found
in the directory `package/rtm/`.

### Get Price test
SQRT does not enable yet to check the method return therefore the test for the `get_price` function is a bit useless
Still, we can check that a call to the method works:
```Rust
#[test]
fn test_get_price() {
    // Same initialisation as before
    let mut test_env = TestEnvironment::new();
    let gumball_blueprint = Box::new(GumballBp {});
    let mut gumball_package = Package::new("tests/gumball_machine/package");
    gumball_package.add_blueprint("gumball", gumball_blueprint);
    test_env.publish_package("gumball", gumball_package);
    test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);

    // Call to the method
    test_env.call_method(GumballMethods::GetPrice).run();
}
```

### Buy Gumball test
To test appropriately the `buy_gumball` method we need to check that the user receives a `gumball` token
when they send enough XRD and that the call panics otherwise.  

The test for the first case would be:
```Rust
#[test]
fn test_buy_gumball() {
    // Same initialisation as before
    let mut test_env = TestEnvironment::new();
    let gumball_blueprint = Box::new(GumballBp {});
    let mut gumball_package = Package::new("tests/gumball_machine/package");
    gumball_package.add_blueprint("gumball", gumball_blueprint);
    test_env.publish_package("gumball", gumball_package);
    test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);

    // We want to compare the amount of XRD owned before and after we buy a gumball
    let xrd_owned_before_call = test_env.amount_owned_by_current("radix");

    // We call the method buy_gumball via the variant BuyGumball with argument dec!(15)
    // With our implementation of the trait Method, it will call the method buy_gumball with a Bucket containing 15 XRD.
    test_env.call_method(GumballMethods::BuyGumball(dec!(15))).run();
    
    let new_amount_xrd_amount = test_env.amount_owned_by_current("radix");

    // We check that the current user indeed received the right amount of gumball tokens
    assert_eq!(test_env.amount_owned_by_current("gumball"), Decimal::one());
    // We check that the user spent more than 1.5 XRD (the check is not very precise because we don't know the gas fees)
    assert!(xrd_owned_before_call - new_amount_xrd_amount > dec!("1.5"));
}
```


We would also want to test the case were the user sends a bucket with not enough XRD tokens. When this happens, the Engine
returns the error `ApplicationError(BucketError(ResourceOperationError(InsufficientBalance)))`, we therefore tell the 
ManifestCall that the test is suppose to fail with this error.
```Rust
#[test]
fn test_buy_gumball_not_enough() {
    // Same initialisation as before
    let mut test_env = TestEnvironment::new();
    let gumball_blueprint = Box::new(GumballBp {});
    let mut gumball_package = Package::new("tests/gumball_machine/package");
    gumball_package.add_blueprint("gumball", gumball_blueprint);
    test_env.publish_package("gumball", gumball_package);
    test_env.new_component("gumball_comp", "gumball", vec![DecimalArg(dec!("1.5"))]);
    
    // Here the command will panic because the method buy_gumball panics when the user does not send enough tokens
    test_env.call_method(GumballMethods::BuyGumball(dec!(1)))
        .should_panic(other_error("ApplicationError(BucketError(ResourceOperationError(InsufficientBalance)))"))
        .run();
}
```





