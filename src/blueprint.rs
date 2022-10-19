pub trait Blueprint
{
    /// Returns the name of the function to instantiate the blueprint as first argument
    /// and a vector of arguments (name, value) to call with
    fn instantiate(&self) -> (String, Vec<String>);

    /// Returns the name of the blueprint
    fn name(&self) -> &str;
}