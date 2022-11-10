use scrypto::math::Decimal;

pub trait Method
{
    /// Returns the name of the blueprint method
    fn name(&self) -> &str;

    /// Returns the arguments of the blueprint method
    fn args(&self) -> Option<Vec<Arg>>;

}

pub enum Arg
{
    /// Bucket with resource to send. The String represents the name of the resource and the Decimal the amount to send
    Bucket(String, Decimal),
    Other(String)
}