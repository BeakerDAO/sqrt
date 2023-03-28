//! Defines what is a Package

use crate::blueprint::Blueprint;
use std::collections::HashMap;

/// Defines a Package to be tested
pub struct Package {
    blueprints: HashMap<String, Box<dyn Blueprint>>,
    address: String,
    path: String,
}

impl Package {
    /// Creates a new `[Package`] from a given path
    ///
    /// # Arguments
    /// * `path` - path to the Scrypto code from the project's root
    pub fn new(path: &str) -> Package {
        Package {
            blueprints: HashMap::new(),
            address: "".to_string(),
            path: String::from(path),
        }
    }

    /// Creates a new [`Package`] from a path and a list of blueprints
    ///
    /// # Arguments
    /// * `path` - path to the Scrypto code from the project's root
    /// * `blueprints` - blueprints associated to the package
    pub fn from(path: &str, blueprints: Vec<(&str, Box<dyn Blueprint>)>) -> Package {
        let mut package = Self::new(path);
        for (name, blueprint) in blueprints {
            package.add_blueprint(name, blueprint)
        }
        package
    }

    /// Adds a new blueprint to the [`Package`]
    ///
    /// # Arguments
    /// * `name` - custom name to give to the blueprint
    /// * `blueprint` - blueprint associated to the package
    pub fn add_blueprint(&mut self, name: &str, blueprint: Box<dyn Blueprint>) {
        if !self.blueprints.contains_key(name) {
            self.blueprints.insert(String::from(name), blueprint);
        } else {
            panic!("A blueprint with this name already exists")
        }
    }

    /// Returns a blueprint associated to the [`Package`]
    ///
    /// # Arguments
    /// * `name` - name given to the blueprint
    pub fn get_blueprint(&self, name: &str) -> Option<&Box<dyn Blueprint>> {
        self.blueprints.get(name)
    }

    pub fn set_address(&mut self, address: String) {
        self.address = address;
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}
