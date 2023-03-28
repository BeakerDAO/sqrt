//! Defines traits to be implemented to declare a new blueprint

/// Trait to implement for a new blueprint
pub trait Blueprint {
    /// Returns the name of the function to instantiate the blueprint
    fn instantiation_name(&self) -> &str;

    /// Returns the name of the blueprint
    fn name(&self) -> &str;

    /// Returns the type of admin badge used by the blueprint
    fn has_admin_badge(&self) -> AdminBadge;
}

/// Defines the type of admin badge used by a blueprint
pub enum AdminBadge {
    Internal,
    External(String),
    None,
}
