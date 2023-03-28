 # The SQRT library
The Scrypto Quick Rtm Testing library is a tool that enables its users to easily generate and use Radix Transaction 
Manifests to test a Scrypto package. The Transaction Manifests are exported in a `rtm` subdirectory located in the 
package directory.
 
# Usage
To be able to use this library for your tests, add the following line to your `[dev-dependencies]`:
```
sqrt = { git = "https://github.com/PointSquare/sqrt" }
```

# Disclaimer
This tool was originally built for personal use and therefore its updates might break previous code.

# Examples
Some simple examples are available in the [test](tests) directory. For bigger projects, a broader example can be found
[here](https://github.com/PointSquare/stoichiometric).

To use the library, you have to implement two traits that will explain it how to instantiate your component and call your methods.
We explain in the following subsections how to do so.

## Test Environments

A `TestEnvironment` deals with all the technicalities of testing your Scrypto package (Components, Resources, Packages, 
Accounts, etc...). It enables to reference accounts, components, resources and components by names instead of addresses.
It is important to note that the names are not case-sensitive. 

## Blueprint Trait

The first trait to implement is the Blueprint trait. It tells SQRT how to instantiate a new component of a blueprint:

```Rust
pub trait Blueprint {
    /// Returns the name of the function to instantiate the blueprint
    fn instantiation_name(&self) -> &str;

    /// Returns the name of the blueprint
    fn name(&self) -> &str;

    /// Returns the type of admin badge used by the blueprint
    fn has_admin_badge(&self) -> AdminBadge;
}
```

For every blueprint, create an empty struct in your test file and implement the `Blueprint` trait for it.  For example:
```Rust
pub struct TestBp {}

impl Blueprint for TestBp { /* Implementation */ }
```

## Method trait 

The other trait to implement is the `Method` trait. It explains SQRT how to call methods for your blueprint: 
```Rust
pub trait Method {
    /// Returns the name of the method
    fn name(&self) -> &str;

    /// Returns the arguments of the method
    fn args(&self) -> Option<Vec<Arg>>;

    /// Returns whether the function needs an admin badge to get called
    fn needs_admin_badge(&self) -> bool;

    /// Returns whether to use a custom manifest name
    fn custom_manifest_name(&self) -> Option<&str>;
}
```
The standard way of implementing the trait is to create an `Enum` with one variant for every method of the blueprint. 
The arguments of the variant will be used to call the method with specific arguments.  
It is up to the user to decide the name and the arguments of the variants, the implementation of the `Method` trait will 
then explain SQRT how to properly transform the variants into proper calls.  
For example:

```Rust
pub enum TestMethods {
    FirstMethod(Decimal),
    SecondMethod(String, String, u8),
    /* ... */
}

impl Method for TestMethods { /* Trait implementation */ }
```

## ManifestCalls

To call a custom or a generated Manifest, SQRT uses a ManifestCall. It gives the user the ability to give more details 
about how to call the method/manifest. Calling a method is done in the following way:
```Rust
let mut test_env = TestEnvironment::new();
test_env.call_method(TestMethods::FirstMethod(dec!(1))).run();
```
Calling a custom Manifest is bit more difficult, the user has to supply a list of environment argument and their value.
It is done in the following way:
```Rust
let mut test_env = TestEnvironment::new();
let mut env_args = vec![];
env_args.push((
    "caller_address".to_string(),
    AccountAddressArg(test_env.get_current_account_name().to_string()),
));
env_args.push((
    "component_address".to_string(),
    ComponentAddressArg(test_env.get_current_component_name().to_string()),
));
test_env.call_custom_manifest("first_method", env_args).run();
```
If the user wants to get the actual manifest that is going to be run, they can add the instruction `output_manifest` to 
the ManifestCall in the following way:
```Rust
let mut test_env = TestEnvironment::new();
let output: String = test_env.call_method(TestMethods::FirstMethod(dec!(0)))
    .output_manifest()
    .run()
```

If a ManifestCall is supposed to fail, the user can add the `should_panic` instruction and supply the expected error to
ManifestCall in the following way:
```Rust
let mut test_env = TestEnvironment::new();
test_env.call_method(TestMethods::FirstMethod(dec!(0)))
    .should_panic(assert_error("You should send a positive amount of tokens"))
    .run();
```

### Error
There are two types of errors:
```Rust
pub enum Error {
    /// States that no error is expected
    Success,
    /// States that an assertion is expected to fail with a given message
    AssertFailed(String),
    /// States that another error should happen
    Other(String)
}
```
Errors should be constructed by the following functions:
```Rust
pub fn assert_fail(error_message: &str) -> Error {}
pub fn other_error(error: &str) -> Error {}
```

## Arguments
The way SQRT deals with methods and functions argument is by the `Arg` enum. The `Arg` enum has the following variants:

| Variant                                     | Scrypto Type                                    | Arguments                                                                                                                            | Example                                                                                                                                              |
|---------------------------------------------|-------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| `Unit`                                      | `()`                                            | N/A                                                                                                                                  | `Arg::Unit`                                                                                                                                          |
| `Bool(bool)`                                | `bool`                                          | `bool`: a boolean                                                                                                                    | `Arg::Bool(true)`                                                                                                                                    |
| `I8(i8)`                                    | `i8`                                            | `i8`: an 8-bit integer                                                                                                               | `Arg::I8(-3)`                                                                                                                                        |
| `I16(i16)`                                  | `i16`                                           | `i16`: a 16-bit integer                                                                                                              | `Arg::I16(-3)`                                                                                                                                       |
| `I32(i32)`                                  | `i32`                                           | `i32`: a 32-bit integer                                                                                                              | `Arg::I32(-3)`                                                                                                                                       |
| `I64(i64)`                                  | `i64`                                           | `i64`: a 64-bit integer                                                                                                              | `Arg::I64(-3)`                                                                                                                                       |
| `I128(i128)`                                | `i128`                                          | `i128`: a 128-bit integer                                                                                                            | `Arg::I128(-3)`                                                                                                                                      |
| `U8(u8)`                                    | `u8`                                            | `u8`: an 8-bit unsigned integer                                                                                                      | `Arg::U8(-3)`                                                                                                                                        |
| `U16(u16)`                                  | `u16`                                           | `u16`: a 16-bit unsigned integer                                                                                                     | `Arg::U16(-3)`                                                                                                                                       |
| `U32(u32)`                                  | `u32`                                           | `u32`: a 32-bit unsigned integer                                                                                                     | `Arg::U32(-3)`                                                                                                                                       |
| `U64(u64)`                                  | `u64`                                           | `u64`: a 64-bit unsigned integer                                                                                                     | `Arg::U64(-3)`                                                                                                                                       |
| `U128(u128)`                                | `u128`                                          | `u128`: a 128-bit unsigned integer                                                                                                   | `Arg::U128(-3)`                                                                                                                                      |
| `StringArg(String)`                         | `String`                                        | `String`: a String                                                                                                                   | `Arg::StringArg(String::from("test"))`                                                                                                               |
| `EnumArg(String, Vec<Arg>)`                 | `Enum`                                          | `String`: a String with the name of the variant<br/> `Vec<Arg>`: arguments of the variant                                            | For the enum ```pub enum Test { Test1(String), Test2(Decimal) }```: <br/> `Arg::EnumArg("Test1".to_string(), vec![Arg::StringArg("ok".to_string()])` |
| `TupleArg(Vec<Arg>)`                        | `Tuple`                                         | `Vec<Arg>`: content of the Tuple as other `Arg`s                                                                                     | `Arg::TupleArg(vec![Arg::I8(-1), Arg::I8(-3)])`                                                                                                      |
| `VecArg(Vec<Arg>)`                          | `Vec`                                           | `Vec<Arg>`: content of the Vec as other `Arg`s                                                                                       | `Arg::VecArg(vec![Arg::I8(-1), Arg::I8(-3)])`                                                                                                        |
| `HashMapArg(HashMap<Arg, Arg>)`             | `HashMap`                                       | `HashMap<Arg>`: content of the Hashmap as other `Arg`s                                                                               | `Arg::HasMapArg(map)` where `map` is an `HashMap<Arg>`                                                                                               |                                                                                                                                                      |
| `PackageAddressArg(String)`                 | `PackageAddress`                                | `String`: name associated to the package in the `TestEnvironment`                                                                    | `Arg::PackageAddressArg("test_pkg".to_string()")`                                                                                                    |
| `ComponentAddressArg(String)`               | `ComponentAddress`                              | `String`: name associated to the component in the `TestEnvironment`                                                                  | `Arg::ComponentAddressArg("test_component".to_string()")`                                                                                            |
| `AccountAddressArg(String)`                 | `ComponentAddress`                              | `String`: name associated to the account in the `TestEnvironment`                                                                    | `Arg::AccountAddressArg("default".to_string()")`                                                                                                     |
| `ResourceAddressArg(String)`                | `ResourceAddress`                               | `String`: name associated to the resource in the `TestEnvironment`                                                                   | `Arg::ResourceAddressArg("radix".to_string()")`                                                                                                      |
| `SystemAddressArg(String)`                  | `SystemAddress`                                 | `String`: address of the system                                                                                                      | `Arg::SystemAddressArg("system_3473a".to_string()")`                                                                                                 |
| `FungibleBucketArg(String, Decimal)`        | `Bucket` containing fungible resources          | `String`: name associated to the fungible resource in `TestEnvironment` <br/> `Decimal`: amount to put in the Bucket                 | `Arg::FungibleBucketArg("radix".to_string(), dec!(10))`                                                                                              |
| `NonFungibleBucketArg(String, Vec<String>)` | `Bucket` containing non fungible resources      | `String`: name associated to the non fungible resource in `TestEnvironment` <br/> `Vec<String>`: ids of the NFR to put in the Bucket | `Arg::NonFungibleBucketArg("test_nfr".to_string(), vec!["1".to_string(), "2".to_string()]`                                                           |
| `FungibleProofArg(String, Decimal)`         | `Proof` of an amount of fungible resource owned | `String`: name associated to the fungible resource in `TestEnvironment` <br/> `Decimal`: amount to make the proof of                 | `Arg::FungibleProofArg("radix".to_string(), dec!(10))`                                                                                               |
| `NonFungibleProofArg(String, Vec<String>)`  | `Proof` of ids of non fungible resource owned   | `String`: name associated to the non fungible resource in `TestEnvironment` <br/> `Vec<String>`: ids of the NFR to make the proof of | `Arg::NonProofBucketArg("test_nfr".to_string(), vec!["1".to_string(), "2".to_string()]`                                                              |
| `Expression(String)`                        | `Expression`                                    | `String`: Manifest expression                                                                                                        | `Arg::Expression("ENTIRE_WORKTOP".to_string())`                                                                                                      |
| `Blob(String)`                              | `Blob`                                          | `String`: blob content                                                                                                               | `Arg::Blob(String::from("<sha256_hash_of_the_blob_contents>"))`                                                                                      |
| `NonFungibleAddressArg(String, Box<Arg>)`   | `NonFungibleAddress`                            | `String`: name associated to the non fungible resource in `TestEnvironment` <br/> `Box<Arg>`: id of the NFR                          | `Arg::NonFungibleAddressArg("test_nfr", Box::new(Arg::U32(1)))`                                                                                      |
| `HashArg(String)`                           | `Hash`                                          | `String`: hash                                                                                                                       | `Arg::HashArg("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".to_string())`                                                       |
| `EcdsaSecp256k1PublicKeyArg(String)`        | `EcdsaSecp256k1PublicKey`                       | `String`: key in hexadecimal                                                                                                         | `Arg::EcdsaSecp256k1PublicKeyArg("<hex>".to_string())`                                                                                               |
| `EcdsaSecp256k1SignatureArg(String)`        | `EcdsaSecp256k1Signature`                       | `String`: key in hexadecimal                                                                                                         | `Arg::EcdsaSecp256k1SignatureArg("<hex>".to_string())`                                                                                               |
| `EddsaEd25519PublicKeyArg(String)`          | `EddsaEd25519PublicKey`                         | `String`: key in hexadecimal                                                                                                         | `Arg::EddsaEd25519PublicKeyArg("<hex>".to_string())`                                                                                                 |
| `EddsaEd25519SignatureArg(String)`          | `EddsaEd25519Signature`                         | `String`: key in hexadecimal                                                                                                         | `Arg::EddsaEd25519SignatureArg("<hex>".to_string())`                                                                                                 |
| `DecimalArg(Decimal)`                       | `Decimal`                                       | `Decimal`: a Decimal                                                                                                                 | `Arg::DecimalArg(dec!(1))`                                                                                                                           |
| `PreciseDecimalArg(PreciseDecimal)`         | `PreciseDecimal`                                | `PreciseDecimal`: a PreciseDecimal                                                                                                   | `Arg::PrecisedDecimalArg(pdec!(2))`                                                                                                                  |
| `NonFungibleIdArg(Box<Arg>)`                | `NonFungibleId`                                 | `Box<Arg>`: a Box to an `Arg` representing a NpnFungibleId                                                                           | `Arg::NonFungibleIdArg(Box::new(Arg::U128(1234567890u128)))`                                                                                         |


# Launch tests
Once the tests are written, use the following command to launch them:

```shell
cargo test -- --test-threads=1
```

 # TODO for version 1.0
 - [ ] Deal with return of blueprint methods
 - [ ] Allow multiple arguments return when instantiating a function
 - [ ] Allow multiple possible instantiation
 - [ ] Deal with states of a blueprint
 - [ ] Deal with values of NFRs
 - [ ] Deal with returns and automatically check how things should have evolved
 - [ ] Automatic implementation of method trait

