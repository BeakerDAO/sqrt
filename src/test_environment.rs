//! Environment for a test

use crate::account::Account;
use crate::blueprint::{AdminBadge, Blueprint};
use crate::component::Component;
use crate::manifest::Manifest;
use crate::manifest_call::ManifestCall;
use crate::method::{Arg, Method};
use crate::package::Package;
use crate::resource_manager::ResourceManager;
use crate::transfer::Deposit;
use crate::utils::{
    create_dir, generated_manifest_exists, run_command, run_manifest, write_manifest,
};
use lazy_static::lazy_static;
use regex::Regex;
use scrypto::prelude::{Decimal, Instant, UtcDateTime};
use std::collections::HashMap;
use std::process::Command;

pub struct TestEnvironment {
    accounts: HashMap<String, Account>,
    packages: HashMap<String, Package>,
    components: HashMap<String, Component>,
    resource_manager: ResourceManager,
    current_account: String,
    current_package: Option<String>,
    current_component: Option<String>,
}

impl TestEnvironment {
    /// Returns a new TestEnvironment
    pub fn new() -> TestEnvironment {
        Self::reset();

        let mut default_account = Account::new();
        let mut resource_manager = ResourceManager::new();
        resource_manager.submit_owner_badge(&mut default_account, "default");
        let mut accounts = HashMap::new();
        accounts.insert(String::from("default"), default_account);

        TestEnvironment {
            accounts,
            packages: HashMap::new(),
            components: HashMap::new(),
            resource_manager,
            current_account: String::from("default"),
            current_package: None,
            current_component: None,
        }
    }

    /// Creates a new account with a given name
    ///
    /// # Arguments
    /// * `name` - name associated to the account
    pub fn create_account(&mut self, name: &str) -> &str {
        let real_name = String::from(name).to_lowercase();
        if self.accounts.contains_key(&real_name) {
            panic!("An account with this name already exists");
        } else {
            let mut new_account = Account::new();
            self.resource_manager
                .submit_owner_badge(&mut new_account, &real_name);
            self.accounts.insert(real_name.clone(), new_account);
            self.accounts.get(&real_name).unwrap().address()
        }
    }

    /// Creates a new token with fixed supply and with a given name
    ///
    /// # Arguments
    /// * `name` - name associated to the token
    /// * `initial_supply` - initial supply for the token
    pub fn create_fixed_supply_token(&mut self, name: &str, initial_supply: Decimal) {
        let name = String::from(name);
        if self.resource_manager.exists(&name) {
            panic!("A token with same name already exists!")
        } else {
            let output = run_command(
                Command::new("resim")
                    .arg("new-token-fixed")
                    .arg(initial_supply.to_string()),
                false,
            );

            lazy_static! {
                static ref ADDRESS_RE: Regex = Regex::new(r#"Resource: (\w*)"#).unwrap();
            }

            let resource_address = String::from(&ADDRESS_RE.captures(&output.0).unwrap()[1]);

            self.resource_manager
                .add_resource(&name, resource_address, true);
            self.update_current_account();
        }
    }


    /// Creates a new token with mutable supply and with a given name
    ///
    /// # Arguments
    /// * `name` - name associated to the token
    /// * `minter_badge` - minter resource address
    pub fn create_mintable_token(&mut self, name: &str, minter_badge: &str) {
        let name = String::from(name);
        if self.resource_manager.exists(&name) {
            panic!("A token with same name already exists!")
        } else {

            let minter_badge = self.resource_manager.get_address(minter_badge);

            let output = run_command(
                Command::new("resim")
                    .arg("new-token-mutable")
                    .arg(minter_badge),
                false,
            );

            lazy_static! {
                static ref ADDRESS_RE: Regex = Regex::new(r#"Resource: (\w*)"#).unwrap();
            }

            let resource_address = String::from(&ADDRESS_RE.captures(&output.0).unwrap()[1]);

            self.resource_manager
                .add_resource(&name, resource_address, true);
            self.update_current_account();
        }

    }

    /// Publishes a new package to resim and the test environment
    ///
    /// # Arguments
    /// * `name` - name associated to the package
    /// * `package` - package to publish
    pub fn publish_package(&mut self, name: &str, mut package: Package) {
        let real_name = String::from(name).to_lowercase();

        if !self.packages.contains_key(&real_name) {
            lazy_static! {
                static ref PACKAGE_RE: Regex = Regex::new(r"Success! New Package: (\w*)").unwrap();
            }

            let package_output = run_command(
                Command::new("resim")
                    .arg("publish")
                    .arg(package.path())
                    .arg("--owner-badge")
                    .arg(self.get_current_account().owner_badge()),
                false,
            );

            let package_address = &PACKAGE_RE.captures(&package_output.0).expect(&format!(
                "Something went wrong! Maybe the path was incorrect? \n{}",
                package_output.0
            ))[1];

            package.set_address(String::from(package_address));
            create_dir(package.path());
            self.packages.insert(real_name.clone(), package);

            if self.current_package.is_none() {
                self.set_current_package(name);
            };
        } else {
            panic!("A package with the same name already exists!");
        }
    }

    /// Creates a new Component of a given blueprint
    ///
    /// When instantiating a new component, newly created resources will be added to the TestEnvironment
    /// # Arguments
    /// * `name` - name associated to the component
    /// * `blueprint_name` - name of the blueprint
    /// * `args_values` - value of the arguments needed to instantiate the Component
    pub fn new_component(&mut self, name: &str, blueprint_name: &str, args: Vec<Arg>) {
        if self.current_package.is_none() {
            panic!("Please create a package first");
        }

        if self.components.contains_key(name) {
            panic!("A component with the same name already exists!")
        }

        let package = self.get_current_package();

        match package.get_blueprint(blueprint_name) {
            Some(box_blueprint) => {
                let blueprint = box_blueprint.as_ref();
                let output = self.instantiate(blueprint, package.path(), package.address(), &args);

                lazy_static! {
                    static ref COMPONENT_RE: Regex = Regex::new(r#"ComponentAddress\("(\w*)"\)"#).unwrap();
                }

                let component_address = &COMPONENT_RE.captures(&output.0).expect(&format!(
                    "Something went wrong when trying to instantiate blueprint! \n{}",
                    output.0
                ))[1];

                let opt_badge: Option<String> = match blueprint.has_admin_badge() {

                    AdminBadge::Internal => {
                        lazy_static! {
                        static ref ADMIN_BADGE: Regex = Regex::new(r#"Resource: (\w*)"#).unwrap();
                    }

                        let badge = &ADMIN_BADGE
                            .captures(&output.0)
                            .expect("Could not read admin badge address!")[1];
                        Some(String::from(badge))
                    }

                    AdminBadge::External(admin_badge_address) => {
                        Some(self.resource_manager.get_address(&admin_badge_address).clone())
                    }

                    AdminBadge::None => { None }

                };

                let comp = Component::from(component_address, package.path(), opt_badge);
                self.components.insert(String::from(name), comp);

                if self.current_component.is_none() {
                    self.set_current_component(name);
                }

                self.resource_manager.update_resources();
                self.update_current_account();
            }
            None => {
                panic!(
                    "Could not find a blueprint named {} for current the package",
                    blueprint_name
                );
            }
        }
    }

    pub fn new_component_from(&mut self, package: &str, component_name: &str, component_address: String, admin_badge_address: Option<String>)
    {
        if self.components.contains_key(component_name) {
            panic!("A component with the same name already exists!")
        }

        let package = self.packages.get(package).unwrap();
        let comp = Component::from(&component_address, package.path(), admin_badge_address);
        self.components.insert(component_name.to_string(), comp);
    }

    /// Creates a [`ManifestCall`] for the given method
    ///
    /// # Arguments
    /// * `method` -  [Method] to call
    pub fn call_method<M>(&mut self, method: M) -> ManifestCall
    where
        M: Method,
    {
        let component_address = self.get_current_component().address().to_string();
        let package_path = self.get_current_package().path().to_string();
        let component_badge = self.get_current_component().admin_badge().clone();
        self.call(method, component_address, package_path, component_badge)
    }

    /// Creates a custom [`ManifestCall`] for the given Manifest
    ///
    /// # Arguments
    /// * `name` -  name of the manifest to call
    /// * `env_args` - vector of (arg_name,value) to replace environment arguments with
    pub fn call_custom_manifest(
        &mut self,
        name: &str,
        env_args: Vec<(String, Arg)>,
    ) -> ManifestCall {
        let (args_name, args): (Vec<String>, Vec<Arg>) = env_args.into_iter().unzip();
        let mut tmp_bindings = vec![];
        self.generate_bindings(&args, &mut tmp_bindings);

        let (_, args_value): (Vec<String>, Vec<String>) = tmp_bindings.into_iter().unzip();
        let mut final_bindings: Vec<(String, String)> =
            args_name.into_iter().zip(args_value).collect();

        ManifestCall::new(self)
            .call_manifest(name, true)
            .add_bindings(&mut final_bindings)
    }

    /// Updates the resources and the current account
    pub fn update(&mut self) {
        self.resource_manager.update_resources();
        self.update_current_account();
    }

    /// Transfers a given amount of tokens from the current account to a given account
    ///
    /// # Arguments
    /// * `account_name` - name associated to the receiver
    /// * `token` -  name associated to the token to transfer
    /// * `amount` - amount of the token to transfer
    pub fn transfer_to(&mut self, account_name: &str, token: &str, amount: Decimal) {
        match self.accounts.get(account_name) {
            None => {
                panic!("Account {} does not exist", account_name)
            }
            Some(acc) => {
                let account_address = acc.address().to_string();
                let owned = self.amount_owned_by_current(token);
                if owned < amount {
                    panic!(
                        "Current account does not own enough token {} (owns {})",
                        token, owned
                    )
                } else {
                    let transfer = Deposit {
                        amount,
                        resource: token.to_string(),
                    };
                    let package_path = self.get_current_package().path().to_string();
                    self.call(transfer, account_address, package_path, None)
                        .run();
                }
            }
        }
    }

    /// Sets the epoch to the given number
    ///
    /// # Arguments
    /// * `epoch` - new epoch
    pub fn set_current_epoch(&mut self, epoch: u64) {
        run_command(
            Command::new("resim")
                .arg("set-current-epoch")
                .arg(epoch.to_string()),
            false,
        );
    }

    /// Sets the current time
    pub fn set_current_time(&mut self, time: Instant) {
        let utc_time = UtcDateTime::from_instant(&time).unwrap();

        run_command(Command::new("resim").arg("set-current-time").arg(format!("{}", utc_time)), false);
    }

    /// Sets the current account to be used
    ///
    /// # Arguments
    /// * `account_name` -  name associated to the account to use as current account
    pub fn set_current_account(&mut self, account_name: &str) {
        let real_name = String::from(account_name).to_lowercase();
        let account = self
            .accounts
            .get(&real_name)
            .expect("Given account does not exist");
        run_command(
            Command::new("resim")
                .arg("set-default-account")
                .arg(account.address())
                .arg(account.private_key())
                .arg(account.owner_badge()),
            false,
        );

        self.current_account = real_name;
    }

    pub fn get_current_account_address(&self) -> &str {
        self.get_current_account().address()
    }

    pub fn get_current_account_name(&self) -> &str {
        &self.current_account
    }

    pub fn get_account_address(&self, name: &str) -> &str {
        self.get_account(name).unwrap().address()
    }

    /// Returns the address of a given Resource
    ///
    /// # Arguments
    /// * `name` -  name associated to the resource
    pub fn get_resource(&self, name: &str) -> &String {
        self.resource_manager.get_address(name)
    }

    /// Returns the amount of a given Resource owned by a given account
    ///
    /// # Arguments
    /// * `account_name` -  name associated to the account
    /// * `resource_name` - name associated to the resource
    pub fn amount_owned_by(&self, account_name: &str, resource_name: &str) -> Decimal {
        match self.accounts.get(account_name) {
            None => {
                panic!("The account {} does not exist", account_name)
            }
            Some(acc) => acc.amount_owned(self.get_resource(resource_name)),
        }
    }

    /// Returns the amount of a given Resource owned by the current account
    ///
    /// # Arguments
    /// * `resource_name` - name associated to the resource
    pub fn amount_owned_by_current(&self, resource_name: &str) -> Decimal {
        self.get_current_account()
            .amount_owned(self.get_resource(resource_name))
    }

    /// Returns the ids owned by a given account for a given Non Fungible Resource
    ///
    /// # Arguments
    /// * `account_name` -  name associated to the account
    /// * `resource_name` - name associated to the resource
    pub fn get_non_fungible_ids_owned_by(
        &self,
        account_name: &str,
        resource_name: &str,
    ) -> Option<&Vec<String>> {
        match self.accounts.get(account_name) {
            None => {
                panic!("The account {} does not exist", account_name)
            }
            Some(acc) => {
                acc.get_non_fungibles_ids(self.resource_manager.get_address(resource_name))
            }
        }
    }

    /// Returns the ids owned by the current account for a given Non Fungible Resource
    ///
    /// # Arguments
    /// * `resource_name` - name associated to the resource
    pub fn get_non_fungible_ids_owned_by_current(&self, resource: &str) -> Option<&Vec<String>> {
        self.get_current_account()
            .get_non_fungibles_ids(self.resource_manager.get_address(resource))
    }

    /// Returns a reference to the current package
    pub fn get_current_package(&self) -> &Package {
        if self.current_package.is_none() {
            panic!("Please publish a package!");
        }

        let current = self.current_package.as_ref().unwrap();
        self.packages.get(current).unwrap()
    }

    /// Sets the current package to be used
    ///
    /// # Arguments
    /// * `package_name` -  name associated to the package to use as current package
    pub fn set_current_package(&mut self, package_name: &str) {
        let real_name = String::from(package_name).to_lowercase();
        match self.packages.get(&real_name) {
            None => {
                panic!("There is no package with name {}", package_name)
            }
            Some(_) => {
                self.current_package = Some(real_name);
            }
        }
    }

    pub fn get_current_package_name(&self) -> Option<&str>
    {
        self.current_package.as_deref()
    }

    /// Returns a reference to the current component
    pub fn get_current_component(&self) -> &Component {
        if self.current_component.is_none() {
            panic!("Please instantiate a component!");
        }

        let current = self.current_component.as_ref().unwrap();
        self.components.get(current).unwrap()
    }

    pub fn get_current_component_name(&self) -> Option<&str> {
        self.current_component.as_deref()
    }

    /// Sets the current component to be used
    ///
    /// # Arguments
    /// * `component_name` -  name associated to the component to use as current component
    pub fn set_current_component(&mut self, component_name: &str) {
        let real_name = String::from(component_name).to_lowercase();
        match self.components.get(&real_name) {
            None => {
                panic!("There is no component with name {}", component_name)
            }
            Some(_) => {
                self.current_component = Some(real_name);
            }
        }
    }

    /// Returns the address of a given component
    ///
    /// # Arguments
    /// * `component_name` -  name associated to the component
    pub fn get_component(&self, component_name: &str) -> Option<&str> {
        match self.components.get(component_name) {
            None => None,
            Some(comp) => Some(comp.address()),
        }
    }

    fn update_current_account(&mut self) {
        let account = self.accounts.get_mut(&self.current_account).unwrap();
        self.resource_manager.update_resources_for_account(account);
    }

    fn get_current_account(&self) -> &Account {
        self.accounts.get(&self.current_account).unwrap()
    }

    fn get_account(&self, name: &str) -> Option<&Account> {
        self.accounts.get(name)
    }

    fn create_instantiation_manifest<B>(path: &str, blueprint: &B, args: &Vec<Arg>) -> String
    where
        B: Blueprint + ?Sized,
    {
        let mut manifest = Manifest::new();
        manifest.instantiate(blueprint, args);
        let manifest_string = manifest.build();
        let name = format!("{}_instantiation", blueprint.name());
        write_manifest(manifest_string, path, name.as_str())
    }

    fn create_method_manifest<M>(path: &str, method: &M)
    where
        M: Method,
    {
        let mut manifest = Manifest::new();
        manifest.call_method(method);
        let manifest_string = manifest.build();
        let manifest_name = match method.custom_manifest_name() {
            None => method.name(),
            Some(name) => name
        };
        write_manifest(manifest_string, path, manifest_name);
    }

    fn get_binding_for(&self, arg: &Arg, arg_count: u32) -> (String, String) {
        let arg_name = format!("arg_{}", arg_count);
        let arg_value = match arg {
            Arg::Unit => {
                panic!("Should not happen")
            }
            Arg::Bool(value) => {
                format!("{}", *value)
            }
            Arg::I8(value) => {
                format!("{}", *value)
            }
            Arg::I16(value) => {
                format!("{}", *value)
            }
            Arg::I32(value) => {
                format!("{}", *value)
            }
            Arg::I64(value) => {
                format!("{}", *value)
            }
            Arg::I128(value) => {
                format!("{}", *value)
            }
            Arg::U8(value) => {
                format!("{}", *value)
            }
            Arg::U16(value) => {
                format!("{}", *value)
            }
            Arg::U32(value) => {
                format!("{}", *value)
            }
            Arg::U64(value) => {
                format!("{}", *value)
            }
            Arg::U128(value) => {
                format!("{}", *value)
            }
            Arg::StringArg(value)
            | Arg::SystemAddressArg(value)
            | Arg::Expression(value)
            | Arg::Blob(value)
            | Arg::HashArg(value)
            | Arg::EcdsaSecp256k1PublicKeyArg(value)
            | Arg::EcdsaSecp256k1Signature(value)
            | Arg::EddsaEd25519PublicKey(value)
            | Arg::EddsaEd25519Signature(value) => {
                format!("{}", *value)
            }

            Arg::EnumArg(variant, fields) => {
                format!("{}u8, {}", variant, self.get_binding_for_elements(fields))
            }
            Arg::TupleArg(elements) | Arg::VecArg(elements) => {
                format!("{}", self.get_binding_for_elements(elements))
            }
            Arg::HashMapArg(elements) => {
                let mut string = String::new();
                for (key_arg, value_arg) in elements {
                    let (_, key_value) = self.get_binding_for(key_arg, 0);
                    let (_, value_value) = self.get_binding_for(value_arg, 0);
                    string = format!("{}Tuple({}, {}), ", string, key_value, value_value);
                }
                string.pop();
                string.pop();
                string
            }
            Arg::PackageAddressArg(name) => match self.packages.get(name) {
                None => {
                    panic!("No package with name {}", name)
                }
                Some(package) => String::from(package.address()),
            },
            Arg::ComponentAddressArg(name) => match self.get_component(&name) {
                None => {
                    panic!("No components with name {}", name)
                }
                Some(comp) => String::from(comp),
            },
            Arg::AccountAddressArg(name) => match self.get_account(&name) {
                None => {
                    panic!("No account with name {}", name)
                }
                Some(account) => String::from(account.address()),
            },

            Arg::ResourceAddressArg(name) => self.resource_manager.get_address(name).clone(),
            Arg::DecimalArg(value) => value.to_string(),
            Arg::PreciseDecimalArg(value) => value.to_string(),
            Arg::NonFungibleLocalId(arg) => {
                let (_, value) = self.get_binding_for(arg.as_ref(), arg_count);
                value
            }
            Arg::FungibleBucketArg(_, _)
            | Arg::NonFungibleBucketArg(_, _)
            | Arg::FungibleProofArg(_, _)
            | Arg::NonFungibleProofArg(_, _) => {
                panic!("This should not happen")
            }
            Arg::NonFungibleGlobalAddress(name, arg) => {
                let (_, id_value) = self.get_binding_for(arg.as_ref(), 0);
                let resource_value = self.resource_manager.get_address(name);

                format!("{}, {}", resource_value, id_value)
            }
        };

        (arg_name, arg_value)
    }

    fn get_binding_for_elements(&self, args: &Vec<Arg>) -> String {
        let mut string = String::new();
        for arg in args {
            let (_, value) = self.get_binding_for(arg, 0);
            let fake_generic = arg.to_generic(0);
            let arg_str = fake_generic.replace("${arg_0}", &value);
            string = format!("{}{}, ", string, arg_str)
        }
        string.pop();
        string.pop();
        string
    }

    fn instantiate<B>(
        &self,
        blueprint: &B,
        package_path: &str,
        package_address: &str,
        args: &Vec<Arg>,
    ) -> (String, String)
    where
        B: Blueprint + ?Sized,
    {
        let name = format!("{}_instantiation", blueprint.name());
        if !generated_manifest_exists(name.as_str(), package_path) {
            Self::create_instantiation_manifest(package_path, blueprint, args);
        }

        let account_comp = String::from(self.get_current_account().address());

        let mut env_binding = vec![];
        env_binding.push((Manifest::caller_arg(), account_comp));
        env_binding.push((Manifest::package_arg(), package_address.to_string()));

        self.generate_bindings(args, &mut env_binding);
        let (_, stdout, stderr) = run_manifest(package_path, name.as_str(), false, env_binding);
        (stdout, stderr)
    }

    fn reset() {
        run_command(Command::new("resim").arg("reset"), false);
    }

    fn call<M>(
        &mut self,
        method: M,
        component_address: String,
        package_path: String,
        component_badge: Option<String>,
    ) -> ManifestCall
    where
        M: Method,
    {
        if !generated_manifest_exists(method.name(), &package_path) {
            Self::create_method_manifest(&package_path, &method);
        }

        let account_comp = String::from(self.get_current_account().address());

        let mut env_binding = vec![];
        env_binding.push((Manifest::caller_arg(), account_comp));
        env_binding.push((Manifest::component_arg(), component_address));
        match component_badge {
            None => {}
            Some(badge) => {
                env_binding.push((Manifest::admin_badge_arg(), badge));
            }
        }
        match method.args() {
            None => {}
            Some(args_vec) => {
                self.generate_bindings(&args_vec, &mut env_binding);
            }
        }

        let manifest_name = match method.custom_manifest_name() {
            None => method.name(),
            Some(name) => name
        };

        ManifestCall::new(self)
            .call_manifest(manifest_name, false)
            .add_bindings(&mut env_binding)
    }

    fn generate_bindings(&self, args: &Vec<Arg>, env_binding: &mut Vec<(String, String)>) {
        let mut arg_count = 0u32;
        for arg in args {
            match arg {
                Arg::Unit => {}
                Arg::FungibleBucketArg(name, amount) => {
                    let resource_arg_name = format!("arg_{}_resource", arg_count);
                    let amount_arg_name = format!("arg_{}_amount", arg_count);
                    env_binding.push((resource_arg_name, self.get_resource(&name).clone()));
                    env_binding.push((amount_arg_name, amount.to_string()));
                }
                Arg::NonFungibleBucketArg(name, ids) => {
                    let resource_arg_name = format!("arg_{}_resource", arg_count);
                    env_binding.push((resource_arg_name, self.get_resource(&name).clone()));

                    let ids_arg_name = format!("arg_{}_ids", arg_count);
                    let mut ids_arg_value = String::new();
                    for id_value in ids {
                        ids_arg_value =
                            format!("{}NonFungibleLocalId(\"{}\") ,", ids_arg_value, id_value);
                    }
                    ids_arg_value.pop();
                    ids_arg_value.pop();
                    env_binding.push((ids_arg_name, ids_arg_value));
                }
                Arg::FungibleProofArg(name, amount) => {
                    let resource_arg_name = format!("arg_{}_resource", arg_count);
                    let amount_arg_name = format!("arg_{}_amount", arg_count);
                    env_binding.push((resource_arg_name, self.get_resource(&name).clone()));
                    env_binding.push((amount_arg_name, format!("Decimal(\"{}\")", amount)));
                }
                Arg::NonFungibleProofArg(name, ids) => {
                    let resource_arg_name = format!("arg_{}_resource", arg_count);
                    env_binding.push((resource_arg_name, self.get_resource(&name).clone()));

                    let ids_arg_name = format!("arg_{}_ids", arg_count);
                    let mut ids_arg_value = String::new();
                    for id_value in ids {
                        ids_arg_value =
                            format!("{}NonFungibleLocalId(\"{}\") ,", ids_arg_value, id_value);
                    }
                    ids_arg_value.pop();
                    ids_arg_value.pop();
                    env_binding.push((ids_arg_name, ids_arg_value));
                }
                _ => env_binding.push(self.get_binding_for(&arg, arg_count)),
            }
            arg_count += 1;
        }
    }
}
