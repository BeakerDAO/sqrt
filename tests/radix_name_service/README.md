# Radix Name Service

We explain here how `SQRT` can be used to test the Radix Name Service repository.
To test the blueprint, we have to implement the `Blueprint` and the `Method` traits.

## Blueprint trait implementation

The purpose of the `Blueprint` trait is to explain to `SQRT` how to instantiate a new component.
In our case, one has to call the function:

```Rust
pub fn instantiate_rns(
    deposit_per_year: Decimal,
    fee_address_update: Decimal,
    fee_renewal_per_year: Decimal,
) -> (ComponentAddress, Bucket) {}
```
Therefore, we make the following implementation for the trait:
```Rust
// To define a Blueprint, we need to implement the Blueprint Trait for some object.
// We therefore define an empty struct for which we will implement the Blueprint Trait.
struct RNSBp {}

impl Blueprint for RNSBp {
    fn instantiate(&self, arg_values: Vec<String>) -> (&str, Vec<String>) {
        // We return the name of the instantiation function and we pass the arguments
        let function_name = "instantiate_rns";
        (function_name, arg_values)
    }

    fn name(&self) -> &str {
        "RadixNameService"
    }

    // The blueprint creates an admin badge, so we tell it to SQRT
    fn has_admin_badge(&self) -> bool {
        true
    }
}
```

We can now instantiate a new blueprint in our tests in the following way:
```Rust
#[test]
fn test_instantiate() {
    // We create a new TestEnvironment. When doing so, a default account is created and is referenced by "default"
    let mut test_env = TestEnvironment::new();
    
    // Create a new instance of RNSBp for which we have implemented the Blueprint trait
    let rns_blueprint = Box::new(RNSBp {});
    
    // We create a new virtual package for the Scrypto package RNS
    let mut rns_package = Package::new("tests/radix_name_service/package/");
    // We add the blueprint "RadixNameService" to the package and we give it the name "rns_bp" so that we can find it later
    rns_package.add_blueprint("rns", rns_blueprint);
    // We can now publish the new package and we give it the name "rns_pkg" so that we can find it easily later
    // This package will be used as the default package
    test_env.publish_package("rns", rns_package);
    
    
    // We can now instantiate a new component (which will be referenced as "rns_comp")
    // We first create a vector with the desired arguments for the instantiation of the blueprint. Here, the arguments 
    // are "deposit_per_year", "fee_address_update" and "fee_renewal_per_year".
    let args = vec![
        String::from("1"),
        String::from("0.01"),
        String::from("0.01"),
    ];
    test_env.new_component("rns_comp", "rns", args);

    // When instantiating a new RNS component, the blueprint creates a new Non Fungible Resource called "DomainName"
    // We check that the new resource has indeed been recognized by the TestEnvironment
    test_env.get_resource("DomainName");
}
```

## Method trait implementation

Now that we can instantiate a `RNS` component, we would like to test its methods.
To be able to test the methods of a blueprint, we need to explain to `SQRT` how we want to call it.  
The standard way of doing so is to create an `Enum` with one variant for every method of the blueprint. The arguments of
the variant will be used to call the method with specific arguments.  
In our case, a `RNS` component has 6 implemented public methods:
```Rust
impl RadixNameService {
    pub fn lookup_address(&self, name: String) -> String {}
    pub fn register_name(
        &mut self,
        name: String,
        target_address: ComponentAddress,
        reserve_years: u8,
        mut deposit: Bucket,
    ) -> (Bucket, Bucket) {}
    pub fn unregister_name(&mut self, name_nft: Bucket) -> Bucket {}
    pub fn update_address(
        &mut self,
        name_nft: Proof,
        new_address: ComponentAddress,
        mut fee: Bucket,
    ) -> Bucket {}
    pub fn renew_name(&mut self, name_nft: Proof, renew_years: u8, mut fee: Bucket) -> Bucket {}
    pub fn withdraw_fees(&mut self) -> Bucket {}
}
```
Because SQRT does not support checking the return of a method yet, we cannot properly test `lookup_address`. SQRT also
does not support getting values of a NFR, so we cannot test `renew_name` properly. 
We therefore create an `Enum` with 4 variants:
```Rust
// There is no restriction on the name of the variants but it is recommended to use a name close to the name of the
// method it is referring to
enum RNSMethods {
    RegisterName(
        // The String name to supply
        String, 
        // The name of the account we want to buy the domain for
        String, 
        // The reserve_year argument to supply
        u8, 
        // The amount of XRD to send
        Decimal),
    
    // The String represent the NFR id to unregister
    UnregisterName(String),
    
    UpdateAddress(
        // The name of the NFR to unregister
        String, 
        // The name of the new account to point to
        String, 
        // The amount of XRD to send
        Decimal),
    
    WithdrawFees,
}
```
The choice of arguments for the variants is up to the user of the library. The implementation of the `Method` trait will
explain how to make the call.  
Here, we made certain decisions but other ones could have worked too.

Implementing the trait `Method` depends on the choice of the variants of the `Enum`. For our choice, the following code
implements the trait:
```Rust
impl Method for RNSMethods {
    
    // We explain here what is the name of the method associated to a variant
    fn name(&self) -> &str {
        match self {
            RNSMethods::RegisterName(_, _, _, _) => "register_name",
            RNSMethods::UnregisterName(_) => "unregister_name",
            RNSMethods::UpdateAddress(_, _, _) => "update_address",
            RNSMethods::WithdrawFees => "withdraw_fees",
        }
    }

    fn args(&self) -> Option<Vec<Arg>> {
        match self {
            
            RNSMethods::RegisterName(name, target_address, reserve_years, deposit_amount) => {
                method_args![
                    // The first expected argument is a String, so we transform the String name to a StringArg
                    StringArg(name.clone()),
                    // The second expected argument is a ComponentAddress. In our case, we will work only with 
                    // ComponentAddresses that represent an account. The AccountAddressArg variant takes an account name
                    // and will find the correct account address.
                    AccountAddressArg(target_address.clone()),
                    // The third expected argument is an u8, so we transform the u8 into a U8 
                    U8(*reserve_years),
                    // The last expected argument is a Bucket containing some XRD as deposit. 
                    // We tell SQRT to take some XRD from the account and create a Bucket with the given XRD by 
                    // creating a FungibleBucketArg with the resource name (here "radix") and with the amount to take
                    FungibleBucketArg(String::from("radix"), *deposit_amount)
                    ]
            }
            RNSMethods::UnregisterName(id) => {
                method_args![
                    // The method unregister_name expects a Bucket containing a DomainName NFR
                    // To tell this to SQRT, we create a NonFungibleBucketArg with the resource name (here "DomainName")
                    // and a vector containing the ids (as Strings) of the DomainName NFR to take.
                    NonFungibleBucketArg(String::from("DomainName"),vec![id.clone()])
                ]
            }
            
            RNSMethods::UpdateAddress(new_address, id, fee) => {
                method_args![
                    // The first expected argument is a Proof of a DomainName NFR. To tell SQRT to create such a Proof,
                    // we create a NonFungibleProofArg with the associated resource name (here "DomainName") and a vector
                    // containing the ids (as Strings) of the DomainName NFRs to create a proof of.
                    NonFungibleProofArg(String::from("DomainName"), vec![id.clone()]),
                    // The second expected argument is a ComponentAddress. In our case, we will work only with 
                    // ComponentAddresses that represent an account. The AccountAddressArg variant takes an account name
                    // and will find the correct account address.
                    AccountAddressArg(new_address.clone()),
                    // The last expected argument is a Bucket containing some XRD as deposit. 
                    // We tell SQRT to take some XRD from the account and create a Bucket with the given XRD by 
                    // creating a FungibleBucketArg with the resource name (here "radix") and with the amount to take
                    FungibleBucketArg(String::from("radix"), *fee)
                    ]
            }
            RNSMethods::WithdrawFees => {
                method_args![]
            }
        }
    }

    // We tell SQRT which methods need a proof of an admin badge. Here only the withdraw_fees method needs one.
    fn needs_admin_badge(&self) -> bool {
        match self {
            RNSMethods::WithdrawFees => true,
            _ => false,
        }
    }
}
```

## Testing the methods

We can now test both methods. Note that a call to a method will create a generic Transaction Manifest that can be found
in the directory `package/rtm/`.


### Register Name test

```Rust
#[test]
fn test_register_name() {
    // Same initialisation as before
    let mut test_env = TestEnvironment::new();
    let rns_blueprint = Box::new(RNSBp {});
    let mut rns_package = Package::new("tests/radix_name_service/package/");
    rns_package.add_blueprint("rns", rns_blueprint);
    test_env.publish_package("rns", rns_package);
    let args = vec![
            String::from("1"),
            String::from("0.01"),
            String::from("0.01"),
        ];
    test_env.new_component("rns_comp", "rns", args);

    
    // We call the register_name method via the RegisterName variant with the following arguments
    test_env.call_method(
        RNSMethods::RegisterName(
            String::from("test.xrd"),
            // The default account name (created when initialising a new TestEnvironment) is "default"
            String::from("default"),
            1,
            dec!("15"),
        ),
    );
    
    let owned_nft = test_env.amount_owned_by_current("DomainName");
    // We check that the account indeed received one DomainName NFR
    assert_eq!(owned_nft, Decimal::one());
}
```

### Unregister Name test

```Rust
#[test]
fn test_unregister() {
    // Same initialisation as before
    let mut test_env = TestEnvironment::new();
    let rns_blueprint = Box::new(RNSBp {});
    let mut rns_package = Package::new("tests/radix_name_service/package/");
    rns_package.add_blueprint("rns", rns_blueprint);
    test_env.publish_package("rns", rns_package);
    let args = vec![
            String::from("1"),
            String::from("0.01"),
            String::from("0.01"),
        ];
    test_env.new_component("rns_comp", "rns", args);

    test_env.call_method(
        RNSMethods::RegisterName(
            String::from("test.xrd"),
            String::from("default"),
            1,
            dec!("15"),
        ),
    );
    let owned_nft = test_env.amount_owned_by_current("DomainName");
    assert_eq!(owned_nft, Decimal::one());

    // We gets the DomainName NFR ids owned by the current account
    let ids = test_env
        .get_non_fungible_ids_owned_by_current("DomainName")
        .unwrap();
    
    // The current account should only own 1 id so we take the first element of the vector ids
    let id = ids.get(0).unwrap();
    
    // We can now call the method unregister_name via the UnregisterName variant with the right DomainName NFR id
    test_env.call_method(RNSMethods::UnregisterName(id.clone()));
    
    let owned_nft = test_env.amount_owned_by_current("DomainName");
    // We check that the current account does not have anymore DomainName NFR.
    assert_eq!(owned_nft, Decimal::zero());
}
```


### Update Address test

```Rust
#[test]
fn test_update_address() {
    let mut test_env = TestEnvironment::new();
    let rns_blueprint = Box::new(RNSBp {});
    let mut rns_package = Package::new("tests/radix_name_service/package/");
    rns_package.add_blueprint("rns", rns_blueprint);
    test_env.publish_package("rns", rns_package);
    let args = vec![
            String::from("1"),
            String::from("0.01"),
            String::from("0.01"),
        ];
    test_env.new_component("rns_comp", "rns", args);

    test_env.call_method(
        RNSMethods::RegisterName(
            String::from("test.xrd"),
            String::from("default"),
            1,
            dec!("15"),
        ),
    );
    let owned_nft = test_env.amount_owned_by_current("DomainName");
    assert_eq!(owned_nft, Decimal::one());

    // We create a new account with name "test"
    // Note that the account does not become the current account by default
    test_env.create_account("test");

    // We get the id of the DomainName NFR owned by the account "default"
    let ids = test_env
        .get_non_fungible_ids_owned_by_current("DomainName")
        .unwrap();
    let id = ids.get(0).unwrap();

    // We can now call the method update_address via the UpdateAddress variant with the right DomainName NFR id
    test_env.call_method(
        RNSMethods::UpdateAddress(String::from("test"), id.clone(), dec!(15)),
    );
    
    // SQRT does not support getting the values of a NFR, so we cannot make assertions on new value of the NFR
}
```


### Withdraw Fees tests

The method withdraw fees should only be callable by users that have the right admin badge.  
We therefore need to make two tests: one with a user having an admin badge and one when the user does not have one.

#### Withdraw Fees with admin badge

```Rust
#[test]
fn test_withdraw_fees() {
    let mut test_env = TestEnvironment::new();
    let rns_blueprint = Box::new(RNSBp {});
    let mut rns_package = Package::new("tests/radix_name_service/package/");
    rns_package.add_blueprint("rns", rns_blueprint);
    test_env.publish_package("rns", rns_package);
    let args = vec![
            String::from("1"),
            String::from("0.01"),
            String::from("0.01"),
        ];
    test_env.new_component("rns_comp", "rns", args);

    test_env.call_method(
        RNSMethods::RegisterName(
            String::from("test.xrd"),
            String::from("default"),
            1,
            dec!("15"),
        ),
    );
    
    let owned_nft = test_env.amount_owned_by_current("DomainName");
    assert_eq!(owned_nft, Decimal::one());

    // This call does not panic therefore it means that everything worked as expected
    test_env.call_method(RNSMethods::WithdrawFees);
}
```

### Withdraw fees with no admin badge

```Rust
#[test]
#[should_panic]
fn test_withdraw_fees_fail() {
    let mut test_env = TestEnvironment::new();
    let rns_blueprint = Box::new(RNSBp {});
    let mut rns_package = Package::new("tests/radix_name_service/package/");
    rns_package.add_blueprint("rns", rns_blueprint);
    test_env.publish_package("rns", rns_package);
    let args = vec![
            String::from("1"),
            String::from("0.01"),
            String::from("0.01"),
        ];
    test_env.new_component("rns_comp", "rns", args);

    test_env.call_method(
        RNSMethods::RegisterName(
            String::from("test.xrd"),
            String::from("default"),
            1,
            dec!("15"),
        ),
    );
    let owned_nft = test_env.amount_owned_by_current("DomainName");
    assert_eq!(owned_nft, Decimal::one());

    // We create another account and set it as current
    test_env.create_account("test");
    test_env.set_current_account("test");

    // As the current account "test" does not have an admin badge, this call will panic as expected
    test_env.call_method(RNSMethods::WithdrawFees);
}
```

