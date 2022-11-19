pub struct Component
{
    address: String,
    package_path: String,
    admin_badge: Option<String>
}

impl Component
{
    pub fn from(address: &str, package_path: &str, admin_badge: Option<String> ) -> Component
    {
        Component {
            address: String::from(address),
            package_path: String::from(package_path),
            admin_badge
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn package_path(&self) -> &str { &self.package_path }

    pub fn admin_badge(&self) -> &Option<String> {
        &self.admin_badge
    }

}