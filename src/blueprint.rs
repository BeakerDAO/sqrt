//! Defines traits to be implemented to declare a new blueprint

/// Trait to implement for a new blueprint
pub trait Blueprint {
    /// Returns the name of the function to instantiate the blueprint as first argument
    /// and a vector of arguments  value to call with
    fn instantiate(&self, arg_values: Vec<String>) -> (&str, Vec<String>);

    /// Returns the name of the blueprint
    fn name(&self) -> &str;

    /// Returns whether the blueprints has an admin badge
    fn has_admin_badge(&self) -> bool;
}
