use std::collections::HashMap;
use crate::blueprint::Blueprint;

pub struct Package
{
    blueprints: HashMap<String, Box<dyn Blueprint>>,
    address: String
}

impl Package
{
    pub fn new() -> Package
    {
        Package
        {
            blueprints: HashMap::new(),
            address: "".to_string()
        }
    }

    pub fn from(blueprints :Vec<(String,Box<dyn Blueprint>)>) -> Package
    {
        let mut package = Self::new();
        for (name,blueprint) in blueprints
        {
            package.add_blueprint(name, blueprint)
        }
        package
    }

    pub fn add_blueprint(&mut self, name: String, blueprint: Box<dyn Blueprint>)
    {
        if !self.blueprints.contains_key(&name)
        {
            self.blueprints.insert(name, blueprint);
        }
        else
        {
            panic!("A blueprint with this name already exists")
        }
    }

    pub fn get_blueprint(&self, name: &str) -> Option<&Box<dyn Blueprint>>
    {
        self.blueprints.get(name)
    }

    pub fn set_address(&mut self, address: String)
    {
        self.address = address;
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}