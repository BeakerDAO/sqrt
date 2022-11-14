use std::collections::HashMap;
use scrypto::math::Decimal;
use scrypto::prelude::{PreciseDecimal};

pub trait Method
{
    /// Returns the name of the blueprint method
    fn name(&self) -> &str;

    /// Returns the arguments of the blueprint method
    fn args(&self) -> Option<Vec<Arg>>;

}

#[derive(Clone)]
pub enum Arg
{
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
    ResultArg(String, String, Box<Result<Arg,Arg>>),
    VecArg(Vec<Arg>),
    HashMapArg(String, String, HashMap<Arg,Arg>),
    DecimalArg(Decimal),
    PreciseDecimalArg(PreciseDecimal),
    PackageAddressArg(String),
    ComponentAddressArg(String),
    ResourceAddressArg(String),
    NonFungibleAddressArg(String),
    HashArg(String),
    /// Bucket with resource to send. The String represents the name of the resource and the Decimal the amount to send
    BucketArg(String, Decimal),
    /// Proof of a resource; second argument tells if proof should be in a bucket
    ProofArg(String),
    NonFungibleIdArg(String)
}


impl Arg
{
    pub fn get_type(&self) -> String
    {
        match self
        {
            Arg::Unit => { format!("()") }
            Arg::Bool(_) => { format!("bool") }
            Arg::I8(_) => { format!("i8") }
            Arg::I16(_) => { format!("i16") }
            Arg::I32(_) => { format!("i32") }
            Arg::I64(_) => { format!("i64") }
            Arg::I128(_) => { format!("i128") }
            Arg::U8(_) => { format!("u8") }
            Arg::U16(_) => { format!("u16") }
            Arg::U32(_) => { format!("u32") }
            Arg::U64(_) => { format!("u64") }
            Arg::U128(_) => { format!("u128") }
            Arg::StringArg(_) => { format!("String") }
            Arg::Struct(name, _) => { name.clone() }
            Arg::OptionArg(name, _) => { format!("Option<{}>", name) }
            Arg::BoxArg(arg) =>
                {
                    let type_name = arg.get_type();
                    format!("Box<{}>", type_name)
                }
            Arg::TupleArg(_) => { format!("Tuple") }
            Arg::ResultArg(ok, err, _) => { format!("Result<{},{}>",ok,err) }
            Arg::VecArg(vec_arg) =>
                {
                    let vec_type = match vec_arg.first()
                    {
                        None => { String::from("()") }
                        Some(arg) => { arg.get_type() }
                    };
                    format!("Vec<{}>", vec_type)
                }
            Arg::HashMapArg(key, value, _) => { format!("Map<{},{}>", key, value) }
            Arg::DecimalArg(_) => { format!("Decimal") }
            Arg::PreciseDecimalArg(_) => { format!("PreciseDecimal") }
            Arg::PackageAddressArg(_) => { format!("PackageAddress") }
            Arg::ComponentAddressArg(_) => { format!("ComponentAddress") }
            Arg::ResourceAddressArg(_) => { format!("ResourceAddress") }
            Arg::NonFungibleAddressArg(_) => { format!("NonFungibleAddress") }
            Arg::HashArg(_) => { format!("Hash") }
            Arg::BucketArg(_, _) => { format!("Bucket") }
            Arg::ProofArg(_) =>
                {
                        format!("Proof")
                }
            Arg::NonFungibleIdArg(_) => { format!("NonFungibleId") }
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