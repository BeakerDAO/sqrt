//! Defines methods that can be called for a blueprint

use std::collections::HashMap;
use radix_engine::types::{Decimal, PreciseDecimal};

/// Trait to implement to declare a new blueprint method
pub trait Method {
    /// Returns the name of the method.
    fn name(&self) -> &str;

    /// Returns the arguments of the method.
    fn args(&self) -> Option<Vec<Arg>>;

    /// Returns whether the function needs an admin badge to be called.
    fn needs_admin_badge(&self) -> bool;

    /// Returns whether to use a custom manifest name.
    fn custom_manifest_name(&self) -> Option<&str>;
}

#[derive(Clone)]
/// Possible arguments for a method call
pub enum Arg {
    Unit,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    StringArg(String),
    /// Enum Argument. The [String] should be the name of the variant of the Enum and the [Vec] the arguments of the variant.
    EnumArg(u8, Vec<Arg>),
    /// Represents a Tuple. The [Vec] should contain the content of the Tuple as other `Arg`s
    TupleArg(Vec<Arg>),
    /// Represents a Vec. The [Vec] should contain the content of the Tuple as other `Arg`s
    VecArg(Vec<Arg>),
    /// Represents a Hashmap.
    HashMapArg(HashMap<Arg, Arg>),
    /// Represents a PackageAddress. The [String] should contain the name of the Package stored by the current TestEnvironment(**NOT** its address)
    PackageAddressArg(String),
    /// Represents a ComponentAddress. The [String] should contain the name of the Component stored by the current TestEnvironment(**NOT** its address)
    ComponentAddressArg(String),
    /// Represents the ComponentAddress of an account. The [String] should contain the name of the Account stored by the current TestEnvironment(**NOT** its address)
    AccountAddressArg(String),
    /// Represents a ResourceAddress. The [String] should contain the name of the Resource stored by the current TestEnvironment(**NOT** its address)
    ResourceAddressArg(String),
    /// Represents a SystemAddress, which address is contained in the [String]
    SystemAddressArg(String),
    /// Represents a Bucket containing some Non Fungible Resource. The [String] should be the name of the resource according to the TestEnvironment (**NOT** the ResourceAddress) and the [Decimal] is the amount to put in the Bucket
    FungibleBucketArg(String, Decimal),
    /// Represents a Bucket containing some Fungible Resource with given ids. The [String] should be the name of the resource according to the TestEnvironment (**NOT** the ResourceAddress) and the [Vec] should contain the ids of the NFR to put inside the Bucket
    NonFungibleBucketArg(String, Vec<String>),
    /// Represents a Proof a Fungible Resource. The [String] should be the name of the resource according to the TestEnvironment (**NOT** the ResourceAddress) and the [Decimal] the amount to use as proof
    FungibleProofArg(String, Decimal),
    /// Represents a Proof a Non-Fungible Resource of given ids. The [String] should be the name of the resource according to the TestEnvironment (**NOT** the ResourceAddress) and the [Vec] should contain the ids of the NFR to build a proof of
    NonFungibleProofArg(String, Vec<String>),
    Expression(String),
    Blob(String),
    NonFungibleGlobalAddress(String, Box<Arg>),
    HashArg(String),
    EcdsaSecp256k1PublicKeyArg(String),
    EcdsaSecp256k1Signature(String),
    EddsaEd25519PublicKey(String),
    EddsaEd25519Signature(String),
    DecimalArg(Decimal),
    PreciseDecimalArg(PreciseDecimal),
    NonFungibleLocalId(Box<Arg>),
}

#[macro_export]
macro_rules! method_args {
    () => (
        None
    );

     ($( $x:expr ),*) => {{
        let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
        Some(temp_vec)
    }};
}

#[macro_export]
macro_rules! enum_arg {

    ($int:expr) => (
        Arg::EnumArg($int, Vec::new())
    );

    ($int:expr, $( $x:expr ),*) => {{
        let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
        Arg::EnumArg($int, temp_vec)
    }};
}

#[macro_export]
macro_rules! tuple_arg {
     ($( $x:expr ),*) => {{
        let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
        Arg::TupleArg(temp_vec)
    }};
}
