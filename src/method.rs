use scrypto::prelude::{Decimal, PreciseDecimal};
use std::collections::HashMap;

pub trait Method {
    /// Returns the name of the blueprint method
    fn name(&self) -> &str;

    /// Returns the arguments of the blueprint method
    fn args(&self) -> Option<Vec<Arg>>;

    /// Return whether the function needs an admin badge to get called
    fn needs_admin_badge(&self) -> bool;
}

#[derive(Clone)]
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
    Struct(String, Vec<Arg>),
    OptionArg(String, Option<Box<Arg>>),
    BoxArg(Box<Arg>),
    TupleArg(Vec<Arg>),
    ResultArg(String, String, Box<Result<Arg, Arg>>),
    VecArg(Vec<Arg>),
    HashMapArg(String, String, HashMap<Arg, Arg>),
    DecimalArg(Decimal),
    PreciseDecimalArg(PreciseDecimal),
    PackageAddressArg(String),
    ComponentAddressArg(String),
    AccountAddressArg(String),
    ResourceAddressArg(String),
    NonFungibleAddressArg(String),
    HashArg(String),
    /// Bucket with resource to send. The String represents the name of the resource and the Decimal the amount to send
    BucketArg(String, Decimal),
    /// Proof of a resource; second argument tells if proof should be in a bucket
    ProofArg(String),
    NonFungibleIdArg(String),
}

impl Arg {
    pub fn get_type(&self) -> String {
        match self {
            Arg::Unit => {
                format!("()")
            }
            Arg::Bool(_) => {
                format!("bool")
            }
            Arg::I8(_) => {
                format!("i8")
            }
            Arg::I16(_) => {
                format!("i16")
            }
            Arg::I32(_) => {
                format!("i32")
            }
            Arg::I64(_) => {
                format!("i64")
            }
            Arg::I128(_) => {
                format!("i128")
            }
            Arg::U8(_) => {
                format!("u8")
            }
            Arg::U16(_) => {
                format!("u16")
            }
            Arg::U32(_) => {
                format!("u32")
            }
            Arg::U64(_) => {
                format!("u64")
            }
            Arg::U128(_) => {
                format!("u128")
            }
            Arg::StringArg(_) => {
                format!("String")
            }
            Arg::Struct(name, _) => name.clone(),
            Arg::OptionArg(name, _) => {
                format!("Option<{}>", name)
            }
            Arg::BoxArg(arg) => {
                let type_name = arg.get_type();
                format!("Box<{}>", type_name)
            }
            Arg::TupleArg(_) => {
                format!("Tuple")
            }
            Arg::ResultArg(ok, err, _) => {
                format!("Result<{},{}>", ok, err)
            }
            Arg::VecArg(vec_arg) => {
                let vec_type = match vec_arg.first() {
                    None => String::from("()"),
                    Some(arg) => arg.get_type(),
                };
                format!("Vec<{}>", vec_type)
            }
            Arg::HashMapArg(key, value, _) => {
                format!("Map<{},{}>", key, value)
            }
            Arg::DecimalArg(_) => {
                format!("Decimal")
            }
            Arg::PreciseDecimalArg(_) => {
                format!("PreciseDecimal")
            }
            Arg::PackageAddressArg(_) => {
                format!("PackageAddress")
            }
            Arg::ComponentAddressArg(_) => {
                format!("ComponentAddress")
            }
            Arg::AccountAddressArg(_) => {
                format!("ComponentAddress")
            }
            Arg::ResourceAddressArg(_) => {
                format!("ResourceAddress")
            }
            Arg::NonFungibleAddressArg(_) => {
                format!("NonFungibleAddress")
            }
            Arg::HashArg(_) => {
                format!("Hash")
            }
            Arg::BucketArg(_, _) => {
                format!("Bucket")
            }
            Arg::ProofArg(_) => {
                format!("Proof")
            }
            Arg::NonFungibleIdArg(_) => {
                format!("NonFungibleId")
            }
        }
    }

    pub fn to_generic(&self, arg_count: u32) -> String
    {
        let generic = format!("${{arg_{}}}", arg_count);
        match self {
            Arg::Unit => { format!("()") }
            Arg::Bool(_) => { generic }
            Arg::I8(_)| Arg::I16(_)| Arg::I32(_)| Arg::I64(_)| Arg::I128(_)| Arg::U8(_)| Arg::U16(_)| Arg::U32(_)| Arg::U64(_)| Arg::U128(_) =>
                {
                    format!("{}{}", generic, self.get_type())
                }
            Arg::StringArg(_) => { format!("\"{}\"", generic) }
            Arg::Struct(_, _) => { format!("Struct({})", generic) }
            Arg::OptionArg(_, _) => { generic }
            Arg::BoxArg(_)|Arg::TupleArg(_) | Arg::ResultArg(_, _, _)| Arg::VecArg(_)| Arg::HashMapArg(_, _, _)  | Arg::HashArg(_) | Arg::BucketArg(_, _) | Arg::ProofArg(_) | Arg::NonFungibleIdArg(_) =>
                {
                    format!("{}({})", self.get_type(), generic)
                }
            Arg::DecimalArg(_) | Arg::PreciseDecimalArg(_) | Arg::PackageAddressArg(_) | Arg::ComponentAddressArg(_) | Arg::AccountAddressArg(_) | Arg::ResourceAddressArg(_) | Arg::NonFungibleAddressArg(_) =>
                {
                    format!("{}(\"{}\")", self.get_type(), generic)
                }
        }
    }

    pub fn value(&self) -> String
    {
        match self {
            Arg::Unit => { format!("()") }
            Arg::Bool(value) => { format!("{}", *value) }
            Arg::I8(int) => { format!("{}", *int)}
            Arg::I16(int) => { format!("{}", *int)}
            Arg::I32(int) => { format!("{}", *int)}
            Arg::I64(int) => { format!("{}", *int)}
            Arg::I128(int) => { format!("{}", *int)}
            Arg::U8(uint) => { format!("{}", *uint) }
            Arg::U16(uint) => { format!("{}", *uint) }
            Arg::U32(uint) => { format!("{}", *uint) }
            Arg::U64(uint) => { format!("{}", *uint) }
            Arg::U128(uint) => { format!("{}", *uint) }
            Arg::StringArg(string) => { format!("{}", string) }
            Arg::Struct(_, _) => { todo!() }
            Arg::OptionArg(_, value) =>
                {
                    match value
                    {
                        None => { String::from("None") }
                        Some(box_arg) => { box_arg.value() }
                    }
                }
            Arg::BoxArg(_)| Arg::TupleArg(_)| Arg::ResultArg(_, _, _)| Arg::VecArg(_)| Arg::HashMapArg(_, _, _) => { todo!() }
            Arg::DecimalArg(value) => { format!("{}", *value) }
            Arg::PreciseDecimalArg(value) => { format!("{}", *value) }
            Arg::PackageAddressArg(_) => { panic!("Should not happen") }
            Arg::ComponentAddressArg(_) => { panic!("Should not happen") }
            Arg::AccountAddressArg(_) => { panic!("Should not happen") }
            Arg::ResourceAddressArg(_) => { panic!("Should not happen") }
            Arg::NonFungibleAddressArg(_) => { panic!("Should not happen") }
            Arg::HashArg(_) => { todo!() }
            Arg::BucketArg(_, _) => { panic!("Should not happen")}
            Arg::ProofArg(_) => { panic!("Should not happen") }
            Arg::NonFungibleIdArg(value) => { format!("{}", *value) }
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
