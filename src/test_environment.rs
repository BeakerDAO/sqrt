use std::collections::HashMap;
use std::path::Path;

use radix_engine::transaction::{CommitResult, TransactionReceipt, TransactionResult};
use radix_engine::types::{Address, ComponentAddress, dec, ManifestValue, PackageAddress, ResourceAddress, ScryptoDecode};
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use transaction::builder::ManifestBuilder;
use transaction::model::TransactionManifest;

use crate::account::Account;
use crate::blueprint::Blueprint;
use crate::formattable::Formattable;
use crate::method_calls::{MethodCallBuilder};
use crate::test_engine::TestEngine;

pub struct TestEnvironment {
    accounts: HashMap<String, Account>,
    components: HashMap<String, ComponentAddress>,
    current_account: String,
    current_component: Option<String>,
    current_package: Option<String>,
    fungibles: HashMap<String, ResourceAddress>,
    non_fungibles: HashMap<String, ResourceAddress>,
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
            components: HashMap::new(),
            current_account: "default".format(),
            current_component: None,
            current_package: None,
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            packages: HashMap::new(),
            test_engine: TestEngine::new(),
        }
    }

    pub fn call_method(&mut self, method_name: &str, args: ManifestValue) -> MethodCallBuilder {
        MethodCallBuilder::from(args, self.current_account().address(), self.current_component().clone(), method_name, &mut self)
    }


    pub fn current_account(&self) -> &Account {
        self.accounts.get(&self.current_account).unwrap()
    }

    pub fn current_component(&self) -> &ComponentAddress { self.components.get(self.current_component.as_ref().unwrap()).unwrap() }

    pub fn current_component_sate<T: ScryptoDecode>(&self) -> T {
        self.test_engine.get_component_state(self.current_component())
    }

    pub fn current_package(&self) -> &PackageAddress { self.packages.get(self.current_package.as_ref().unwrap()).unwrap() }

    pub fn execute_call(&mut self, manifest: TransactionManifest, with_trace: bool) -> TransactionReceipt {
        let receipt = self.test_engine.execute_manifest(manifest, vec![], with_trace);
        if let TransactionResult::Commit(commit_result) = &receipt.result {
            self.update_resources_from_result(commit_result);
        }
        receipt
    }

    pub fn exists_resource<F: Formattable>(&self, name: F) -> bool {
        match self.fungibles.get(&name.format()) {
            None => self.non_fungibles.contains_key(&name.format()),
            Some(_) => true,
        }
    }

    pub fn get_component_state<T: ScryptoDecode, F: Formattable>(&self, component_name: F) -> T {
        let component_address = self.get_component(component_name);
        self.test_engine.get_component_state(component_address)
    }

    pub fn get_current_account_address(&self) -> ComponentAddress
    {
        self.accounts.get(&self.current_account).unwrap().address()
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

    pub fn new_component<F: Formattable, B: Blueprint>(&mut self, component_name: F, blueprint: B, args: ManifestValue)
    {
        match self.components.get(&component_name.format())
        {
            Some(_) => {
                panic!("A component with name {} already exists", component_name.format())
            }
            None => {
                let package_address = self.current_package().clone();
                let current_account = self.get_current_account_address();
                let manifest = ManifestBuilder::new()
                    .lock_fee(current_account, dec!(10))
                    .call_function(package_address, blueprint.name(), blueprint.instantiation_name(), args)
                    .build();

                let receipt = self.test_engine.execute_manifest(manifest, vec![], false);

                if let TransactionResult::Commit(commit) = receipt.result {
                    let component: ComponentAddress = commit.output(1);
                    self.components.insert(component_name.format(), component);

                    let admin_badge_position = blueprint.admin_badge_type().return_position();
                    if admin_badge_position > 0 {
                        let admin_badge: ResourceAddress = commit.output(admin_badge_position);
                        let admin_badge_name = format!("{}_admin_badge", component_name.format()).format();
                        self.fungibles.insert(admin_badge_name, admin_badge);
                    }

                    if self.current_component.is_none() { self.current_component = Some(component_name.format()) };

                    self.update_resources_from_result(&commit);
                }
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
                self.packages.insert(formatted_name.clone(), new_package);
                if self.current_package.is_none() { self.current_package = Some(formatted_name)}
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
    ) {
        let formatted_name = name.format();
        match self.packages.get(&formatted_name) {
            Some(_) => {
                panic!("A package with name {} already exists", formatted_name)
            }
            None => {
                let new_package = self
                    .test_engine
                    .publish_package_with_owner(path, self.current_account().owner_badge());
                self.packages.insert(formatted_name, new_package);
            }
        }
    }

    pub fn set_current_account<F: Formattable>(&mut self, name: F) {
        self.current_account = self.get_account(name);
    }

    fn get_account<F: Formattable>(&self, name: F) -> String {
        match self.accounts.get(&name.format()) {
            None => {
                panic!("There is no account with name {}", name.format())
            }
            Some(_) => name.format(),
        }
    }

    fn get_component<F: Formattable>(&self, name: F) -> &ComponentAddress {
        match self.components.get(&name.format()) {
            None => {
                panic!("There is no component with name {}", name.format())
            }
            Some(address) => address
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

    fn get_non_fungible<F: Formattable>(&self, name: F) -> &ResourceAddress {
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

    fn update_resources_from_result(&mut self, result: &CommitResult) {

        // Update tracked resources
        for resource in result.new_resource_addresses() {

            match self.test_engine.get_metadata(Address::Resource(resource.clone()), "name") {
                None =>
                    {
                        println!("Could not find name for resource {:?}", resource.clone());
                    }
                Some(entry) =>
                    {
                        match entry {
                            MetadataEntry::Value(value) =>
                                {
                                    match value {
                                        MetadataValue::String(name) => {
                                            match resource {
                                                ResourceAddress::Fungible(_) => {
                                                    self.fungibles.insert(name.format(), resource.clone());
                                                }
                                                ResourceAddress::NonFungible(_) => {
                                                    self.non_fungibles.insert(name.format(), resource.clone());
                                                }
                                            }
                                        }
                                        _ => {
                                            println!("Could not find name for resource {:?}", resource.clone());
                                        }
                                    }
                                }
                            MetadataEntry::List(_) => {
                                println!("Could not find name for resource {:?}", resource.clone());
                            }
                        }
                    }
            };
        }
    }
}
