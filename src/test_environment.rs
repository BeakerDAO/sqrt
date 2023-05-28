use crate::account::Account;
use crate::formattable::Formattable;
use crate::test_engine::TestEngine;
use radix_engine::types::{PackageAddress, ResourceAddress};
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use std::collections::HashMap;
use std::path::Path;

pub struct TestEnvironment {
    accounts: HashMap<String, Account>,
    current_account: String,
    fungibles: HashMap<String, ResourceAddress>,
    non_fungibles: HashMap<String, NonFungibleGlobalId>,
    packages: HashMap<String, PackageAddress>,
    test_engine: TestEngine,
}

impl TestEnvironment {
    /// Returns a new test environment with default configuration.
    pub fn new() -> Self {
        let mut test_engine = TestEngine::new();
        let default_account = test_engine.new_account();
        let mut accounts = HashMap::new();
        accounts.insert(String::from("default".format()), default_account);

        Self {
            accounts: HashMap::new(),
            current_account: "default".format(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            packages: HashMap::new(),
            test_engine: TestEngine::new(),
        }
    }

    pub fn current_account(&self) -> &Account {
        self.accounts.get(&self.current_account).unwrap()
    }

    pub fn set_current_account<F: Formattable>(&mut self, name: F) {
        self.current_account = self.get_account(name);
    }

    /// Creates a new account with a given name.
    ///
    /// # Arguments
    /// * `name` - name associated to the account.
    pub fn new_account<F: Formattable>(&mut self, name: F) {
        let formatted_name = name.format();
        match self.accounts.get(&formatted_name) {
            Some(_) => {
                panic!("An account with name {} already exists", formatted_name)
            }
            None => {
                let new_account = self.test_engine.new_account();
                self.accounts.insert(formatted_name, new_account);
            }
        }
    }

    /// Publishes a new package.
    ///
    /// # Arguments
    /// * `name` - name associated to the account.
    /// * `path` - path to the directory of the package.
    pub fn new_package<F: Formattable, P: AsRef<Path>>(&mut self, name: F, path: P) {
        let formatted_name = name.format();
        match self.packages.get(&formatted_name) {
            Some(_) => {
                panic!("A package with name {} already exists", formatted_name)
            }
            None => {
                let new_package = self.test_engine.publish_package(path);
                self.packages.insert(formatted_name, new_package);
            }
        }
    }

    /// Publishes a new package with an owner.
    ///
    /// # Arguments
    /// * `name` - name associated to the account.
    /// * `path` - path to the directory of the package.
    /// * `owner_badge` - name of the non-fungible resource to use as owner badge.
    pub fn new_package_with_owner<F: Formattable, P: AsRef<Path>>(
        &mut self,
        name: F,
        path: P,
        owner_badge: F,
    ) {
        let formatted_name = name.format();
        match self.packages.get(&formatted_name) {
            Some(_) => {
                panic!("A package with name {} already exists", formatted_name)
            }
            None => {
                let owner_badge = self.get_non_fungible(owner_badge);
                let new_package = self
                    .test_engine
                    .publish_package_with_owner(path, owner_badge.clone());
                self.packages.insert(formatted_name, new_package);
            }
        }
    }

    fn get_account<F: Formattable>(&self, name: F) -> String {
        match self.accounts.get(&name.format()) {
            None => {
                panic!("There is no account with name {}", name.format())
            }
            Some(_) => name.format(),
        }
    }

    fn get_fungible<F: Formattable>(&self, name: F) -> &ResourceAddress {
        match self.fungibles.get(&name.format()) {
            None => {
                panic!("There is no fungible resource with name {}", name.format())
            }
            Some(address) => address,
        }
    }

    fn get_non_fungible<F: Formattable>(&self, name: F) -> &NonFungibleGlobalId {
        match self.non_fungibles.get(&name.format()) {
            None => {
                panic!(
                    "There is no non-fungible resource with name {}",
                    name.format()
                )
            }
            Some(address) => address,
        }
    }

    fn get_package<F: Formattable>(&self, name: F) -> &PackageAddress {
        match self.packages.get(&name.format()) {
            None => {
                panic!("There is no package with name {}", name.format())
            }
            Some(address) => address,
        }
    }
}
