# Hello Token 

We explain here how `SQRT` can be used to test the Hello Token repository.
To test the blueprint, we have to implement the `Blueprint` and the `Method` traits.

## Blueprint trait implementation

The purpose of the `Blueprint` trait is to explain to `SQRT` how to instantiate a new component.
In our case, one has to call the function:

```Rust
pub fn instantiate_hello() -> ComponentAddress {}
```
Therefore, we make the following implementation for the trait:

```Rust
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
```

We can now instantiate a new blueprint in our tests in the following way:
```Rust
#[test]
fn test_instantiate() {
    // We create a new TestEnvironment. When doing so, a default account is created and is referenced by "default"
    let mut test_env = TestEnvironment::new();
    
    // Create a new instance of HelloBp for which we have implemented the Blueprint trait
    let hello_blueprint = Box::new(HelloBp {});
    
    // We create a new virtual package for the Scrypto package Hello
    let mut hello_package = Package::new("tests/hello_token/package/");
    // We add the blueprint "Hello" to the package and we give it the name "hello_bp" so that we can find it later
    hello_package.add_blueprint("hello_bp", hello_blueprint);
    // We can now publish the new package and we give it the name "hello_pkg" so that we can find it easily later
    // This package will be used as the default package
    test_env.publish_package("hello_pkg", hello_package);
    
    // We can now instantiate a new component (which will be referenced as "hello_comp") with no arguments 
    test_env.new_component("hello_comp", "hello", vec![]);

    // When instantiating a new Hello component, the blueprint creates two tokens named "HelloTokens" and "test"
    // We check that the new tokens have indeed been recognized by the TestEnvironment
    test_env.get_resource("HelloToken");
    test_env.get_resource("test");
}
```

## Method trait implementation

Now that we can instantiate a `Hello` component, we would like to test its methods.
To be able to test the methods of a blueprint, we need to explain to `SQRT` how we want to call it.  
The standard way of doing so 
is to create an `Enum` with one variant for every method of the blueprint. The arguments of the variant will be used to call
the method with specific arguments.  
In our case, a `Hello` component has only one method:
```Rust
 impl Hello {
    pub fn free_token(&mut self) -> Bucket {}
}
```

We therefore create an `Enum` with one variant which has no argument (because the method has no arguments):
```Rust
// There is no restriction on the name of the variants but it is recommended to use a name close to the name of the
// method it is referring to
 enum HelloMethods {
        FreeToken,
    }
```

For this easy case, the `Method` trait implementation is pretty straightforward:
```Rust
impl Method for HelloMethods {
    
    // We explain here what is the name of the method associated to a variant
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
```


We can finally test the `free_token` method and make sure that it returns the right amount of tokens:

```Rust
#[test]
fn test_free_token() {
    // Same instantiation as earlier
    let mut test_env = TestEnvironment::new();
    let hello_blueprint = Box::new(HelloBp {});
    let mut hello_package = Package::new("tests/hello_token/package/");
    hello_package.add_blueprint("hello", hello_blueprint);
    test_env.publish_package("hello", hello_package);
    test_env.new_component("hello_comp", "hello", vec![]);

    // We call the method FreeToken
    test_env.call_method(HelloMethods::FreeToken);
    // We check that we indeed received 1 HelloToken after having called the FreeToken function
    assert_eq!(test_env.amount_owned_by_current("HelloToken"), Decimal::ONE);
}
```