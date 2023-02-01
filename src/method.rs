//! Defines methods that can be called for a blueprint

use scrypto::prelude::{Decimal, PreciseDecimal};
use std::collections::HashMap;

/// Trait to implement to declare a new blueprint method
pub trait Method {
    /// Returns the name of the method
    fn name(&self) -> &str;

    /// Returns the arguments of the method
    fn args(&self) -> Option<Vec<Arg>>;

    /// Return whether the function needs an admin badge to get called
    fn needs_admin_badge(&self) -> bool;
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
    EnumArg(String, Vec<Arg>),
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
    /// Represents a Proof a Non Fungible Resource of given ids. The [String] should be the name of the resource according to the TestEnvironment (**NOT** the ResourceAddress) and the [Vec] should contain the ids of the NFR to build a proof of
    NonFungibleProofArg(String, Vec<String>),
    Expression(String),
    Blob(String),
    NonFungibleAddressArg(String, Box<Arg>),
    HashArg(String),
    EcdsaSecp256k1PublicKeyArg(String),
    EcdsaSecp256k1Signature(String),
    EddsaEd25519PublicKey(String),
    EddsaEd25519Signature(String),
    DecimalArg(Decimal),
    PreciseDecimalArg(PreciseDecimal),
    NonFungibleIdArg(Box<Arg>),
}

impl Arg {
    /// Returns the type of an `Arg` according to Transaction Manifests
    pub fn get_type(&self) -> String {
        match self {
            Arg::Unit => String::from("()"),
            Arg::Bool(_) => String::from("bool"),
            Arg::I8(_) => String::from("i8"),
            Arg::I16(_) => String::from("i16"),
            Arg::I32(_) => String::from("i32"),
            Arg::I64(_) => String::from("i64"),
            Arg::I128(_) => String::from("i128"),
            Arg::U8(_) => String::from("u8"),
            Arg::U16(_) => String::from("u16"),
            Arg::U32(_) => String::from("u32"),
            Arg::U64(_) => String::from("u64"),
            Arg::U128(_) => String::from("u128"),
            Arg::StringArg(_) => String::from("String"),
            Arg::EnumArg(_, _) => String::from("Enum"),
            Arg::TupleArg(_) => String::from("Tuple"),
            Arg::VecArg(vec_arg) => {
                let vec_type = match vec_arg.first() {
                    None => String::from("()"),
                    Some(arg) => arg.get_type(),
                };
                format!("Array<{}>", vec_type)
            }
            Arg::HashMapArg(_) => String::from("Array<Tuple>"),
            Arg::PackageAddressArg(_) => String::from("PackageAddress"),
            Arg::ComponentAddressArg(_) => String::from("ComponentAddress"),
            Arg::AccountAddressArg(_) => String::from("ComponentAddress"),
            Arg::ResourceAddressArg(_) => String::from("ResourceAddress"),
            Arg::SystemAddressArg(_) => String::from("SystemAddress"),
            Arg::FungibleBucketArg(_, _) => String::from("Bucket"),
            Arg::NonFungibleBucketArg(_, _) => String::from("Bucket"),
            Arg::FungibleProofArg(_, _) => String::from("Proof"),
            Arg::NonFungibleProofArg(_, _) => String::from("Proof"),
            Arg::Expression(_) => String::from("Expression"),
            Arg::Blob(_) => String::from("Blob"),
            Arg::NonFungibleAddressArg(_, _) => String::from("NonFungibleAddress"),
            Arg::HashArg(_) => String::from("Hash"),
            Arg::EcdsaSecp256k1PublicKeyArg(_) => String::from("EcdsaSecp256k1PublicKey"),
            Arg::EcdsaSecp256k1Signature(_) => String::from("EcdsaSecp256k1Signature"),
            Arg::EddsaEd25519PublicKey(_) => String::from("EddsaEd25519PublicKey"),
            Arg::EddsaEd25519Signature(_) => String::from("EddsaEd25519Signature"),
            Arg::DecimalArg(_) => String::from("Decimal"),
            Arg::PreciseDecimalArg(_) => String::from("PreciseDecimal"),
            Arg::NonFungibleIdArg(_) => String::from("NonFungibleId"),
        }
    }

    /// Returns the generic form of an `Arg` for a Transaction Manifest
    pub fn to_generic(&self, arg_count: u32) -> String {
        let generic = format!("${{arg_{}}}", arg_count);
        match self {
            Arg::Unit => String::from("()"),
            Arg::Bool(_) => generic,
            Arg::I8(_)
            | Arg::I16(_)
            | Arg::I32(_)
            | Arg::I64(_)
            | Arg::I128(_)
            | Arg::U8(_)
            | Arg::U16(_)
            | Arg::U32(_)
            | Arg::U64(_)
            | Arg::U128(_) => {
                format!("{}{}", generic, self.get_type())
            }
            Arg::StringArg(_) => {
                format!("\"{}\"", generic)
            }
            Arg::EnumArg(_, _)
            | Arg::TupleArg(_)
            | Arg::VecArg(_)
            | Arg::HashMapArg(_)
            | Arg::FungibleBucketArg(_, _)
            | Arg::NonFungibleBucketArg(_, _)
            | Arg::FungibleProofArg(_, _)
            | Arg::NonFungibleProofArg(_, _) => {
                format!("{}({})", self.get_type(), generic)
            }
            Arg::PackageAddressArg(_)
            | Arg::ComponentAddressArg(_)
            | Arg::AccountAddressArg(_)
            | Arg::ResourceAddressArg(_)
            | Arg::SystemAddressArg(_)
            | Arg::Expression(_)
            | Arg::Blob(_)
            | Arg::NonFungibleAddressArg(_, _)
            | Arg::HashArg(_)
            | Arg::EcdsaSecp256k1PublicKeyArg(_)
            | Arg::EcdsaSecp256k1Signature(_)
            | Arg::EddsaEd25519PublicKey(_)
            | Arg::EddsaEd25519Signature(_)
            | Arg::DecimalArg(_)
            | Arg::PreciseDecimalArg(_) => {
                format!("{}(\"{}\")", self.get_type(), generic)
            }
            Arg::NonFungibleIdArg(arg) => {
                format!("{}({})", self.get_type(), arg.to_generic(arg_count))
            }
        }
    }
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
