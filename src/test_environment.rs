use crate::account::Account;
use crate::component::Component;
use crate::manifest::Manifest;
use crate::method::{Arg, Method};
use crate::package::Package;
use crate::utils::{create_dir, manifest_exists, run_command, run_manifest, transfer, write_manifest, write_transfer};
use crate::RADIX_TOKEN;
use lazy_static::lazy_static;
use regex::Regex;
use scrypto::prelude::{Decimal};
use std::collections::HashMap;
use std::process::Command;
use crate::resource_manager::ResourceManager;

pub struct TestEnvironment {
    accounts: HashMap<String, Account>,
    packages: HashMap<String, Package>,
    components: HashMap<String, Component>,
    current_account: String,
    resource_manager: ResourceManager,
}

impl TestEnvironment {
    pub fn new() -> TestEnvironment {
        Self::reset();

        let mut default_account = Account::new();
        let mut resource_manager = ResourceManager::new();
        resource_manager.generate_owner_badge(&mut default_account);
        let mut accounts = HashMap::new();
        accounts.insert(String::from("default"), default_account);
        let mut tokens = HashMap::new();
        tokens.insert(String::from("radix"), String::from(RADIX_TOKEN));

        TestEnvironment {
            accounts,
            packages: HashMap::new(),
            components: HashMap::new(),
            current_account: String::from("default"),
            resource_manager,
        }
    }

    pub fn create_account(&mut self, name: &str) -> &str {
        let real_name = String::from(name).to_lowercase();
        if self.accounts.contains_key(&real_name) {
            panic!("An account with this name already exists");
        } else {
            self.accounts.insert(real_name.clone(), Account::new());
            self.accounts.get(&real_name).unwrap().address()
        }
    }

    pub fn create_fixed_supply_token(&mut self, name: &str, initial_supply: Decimal) {
        let name = String::from(name);
        if self.resource_manager.exists(&name) {
                panic!("A token with same name already exists!")
            }
            else {
                let output = run_command(
                    Command::new("resim")
                        .arg("new-token-fixed")
                        .arg(initial_supply.to_string()),
                    false,
                );

                lazy_static! {
                    static ref ADDRESS_RE: Regex = Regex::new(r#"Resource: (\w*)"#).unwrap();
                }

                let resource_address = String::from(&ADDRESS_RE.captures(&output).unwrap()[1]);

                self.resource_manager.add_resource(&name, resource_address, true);
                self.update_current_account();
            }

    }

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
                    .arg(self.resource_manager.get_owner_badge()),
                false,
            );

            let package_address = &PACKAGE_RE.captures(&package_output).expect(&format!(
                "Something went wrong! Maybe the path was incorrect? \n{}",
                package_output
            ))[1];

            package.set_address(String::from(package_address));
            create_dir(package.path());
            self.packages.insert(real_name, package);
        } else {
            panic!("A package with the same name already exists!");
        }
    }

    pub fn new_component(
        &mut self,
        name: &str,
        package_name: &str,
        blueprint_name: &str,
        args_values: Vec<String>,
    ) {
        if self.components.contains_key(name) {
            panic!("A component with the same name already exists!")
        }

        match self.packages.get(package_name) {
            Some(package) => match package.get_blueprint(blueprint_name) {
                Some(box_blueprint) => {
                    let blueprint = box_blueprint.as_ref();
                    let (inst_name, args) = blueprint.instantiate(args_values);

                    let output = run_command(
                        Command::new("resim")
                            .arg("call-function")
                            .arg(package.address())
                            .arg(blueprint.name())
                            .arg(inst_name)
                            .args(args),
                        false,
                    );

                    lazy_static! {
                        static ref COMPONENT_RE: Regex = Regex::new(r#"Component: (\w*)"#).unwrap();
                    }

                    let component_address = &COMPONENT_RE.captures(&output).expect(&format!(
                        "Something went wrong when trying to instantiate blueprint! \n{}",
                        output
                    ))[1];

                    let opt_badge: Option<String> = if blueprint.has_admin_badge() {
                        lazy_static! {
                            static ref ADMIN_BADGE: Regex =
                                Regex::new(r#"Resource: (\w*)"#).unwrap();
                        }

                        let badge = &ADMIN_BADGE
                            .captures(&output)
                            .expect("Could not read admin badge address!")[1];
                        Some(String::from(badge))
                    } else {
                        None
                    };

                    let comp = Component::from(component_address, package.path(), opt_badge);
                    self.components.insert(String::from(name), comp);
                    write_transfer(package.path());
                    self.resource_manager.update_resources();
                    self.update_current_account();
                }
                None => {
                    panic!(
                        "Could not find a blueprint named {} for the package {}",
                        blueprint_name, package_name
                    );
                }
            },
            None => {
                panic!("Could not find a package named {}", name);
            }
        }
    }

    pub fn call_method<M>(&mut self, component: &str, method: M)
    where
        M: Method,
    {
        self.call_method_with_output(component, method);
    }

    pub fn call_method_with_output<M>(&mut self, component: &str, method: M) -> String
    where
        M: Method,
    {
        let output;
        match self.components.get(component) {
            None => {
                panic!("No component with name {}", component)
            }
            Some(comp) => {

                if !manifest_exists(method.name(), comp.package_path())
                {
                    Self::create_manifest(comp.package_path(), &method);
                }
                let component_address = String::from(comp.address());
                let account_comp = String::from(self.get_current_account().address());

                let mut env_binding = vec![];
                env_binding.push((Manifest::caller_arg(), account_comp));
                env_binding.push((Manifest::component_arg(), component_address));
                if method.needs_admin_badge() {
                    env_binding.push((Manifest::admin_badge_arg(), comp.admin_badge().as_ref().unwrap().clone()))
                }

                let mut arg_count = 0u32;
                match method.args()
                {
                    None => {}
                    Some(args) => {
                        for arg in args
                        {
                            match arg
                            {
                                Arg::Unit => {}
                                Arg::FungibleBucketArg(name, amount) =>
                                    {
                                        let resource_arg_name = format!("arg_{}_resource",arg_count);
                                        let amount_arg_name = format!("arg_{}_amount", arg_count);
                                        env_binding.push((resource_arg_name, self.get_token(&name).clone()));
                                        env_binding.push((amount_arg_name, amount.to_string()));
                                    }
                                Arg::NonFungibleBucketArg(name, ids) =>
                                    {
                                        let resource_arg_name = format!("arg_{}_resource", arg_count);
                                        env_binding.push((resource_arg_name, self.get_token(&name).clone()));

                                        let ids_arg_name = format!("arg_{}_ids", arg_count);
                                        let mut ids_arg_value = String::new();
                                        for id_value in ids
                                        {
                                            ids_arg_value = format!("{}NonFungibleId({}) ,", ids_arg_value, id_value);
                                        }
                                        ids_arg_value.pop();
                                        ids_arg_value.pop();
                                        env_binding.push((ids_arg_name, ids_arg_value));
                                    }
                                Arg::FungibleProofArg(name, amount) =>
                                    {
                                        let resource_arg_name = format!("arg_{}_resource",arg_count);
                                        let amount_arg_name = format!("arg_{}_amount", arg_count);
                                        env_binding.push((resource_arg_name, self.get_token(&name).clone()));
                                        env_binding.push((amount_arg_name, format!("Decimal(\"{}\")", amount)));
                                    }
                                Arg::NonFungibleProofArg(name, ids) =>
                                    {
                                        let resource_arg_name = format!("arg_{}_resource",arg_count);
                                        env_binding.push((resource_arg_name, self.get_token(&name).clone()));

                                        let ids_arg_name = format!("arg_{}_ids", arg_count);
                                        let mut ids_arg_value = String::new();
                                        for id_value in ids
                                        {
                                            ids_arg_value = format!("{}NonFungibleId({}) ,", ids_arg_value, id_value);
                                        }
                                        ids_arg_value.pop();
                                        ids_arg_value.pop();
                                        env_binding.push((ids_arg_name, ids_arg_value));
                                    }
                                _ => { env_binding.push(self.get_binding_for(&arg, arg_count)) }
                            }
                            arg_count+=1;
                        }
                    }
                }

                output = run_manifest(comp.package_path(), method.name(), env_binding);

                self.resource_manager.update_resources();
                self.update_current_account();
            }
        }

        output
    }

    pub fn transfer_to(&mut self, account: &str, token: &str, amount: Decimal) {
        let from = String::from(self.get_current_account().address());
        match self.accounts.get_mut(account) {
            None => {
                panic!("Account {} does not exist", account)
            }
            Some(acc) =>
                {
                    let resource_address = self.resource_manager.get_address(token);
                    let owned = acc.amount_owned(resource_address);
                    if owned < amount
                        {
                            panic!("Current account does not own enough token {} (owns {})", token, owned)
                        }
                        else {
                            transfer(
                                &from,
                                acc.address(),
                                resource_address,
                                amount.to_string().as_str(),
                            );
                            self.resource_manager.update_resources_for_account(acc);
                            self.update_current_account();
                        }
                    }
                }
    }


    pub fn reset() {
        run_command(Command::new("resim").arg("reset"), false);
    }

    fn update_current_account(&mut self)
    {
        let account = self.accounts.get_mut(&self.current_account).unwrap();
        self.resource_manager.update_resources_for_account(account);
    }

    pub fn set_current_epoch(&mut self, epoch: u64) {
        run_command(
            Command::new("resim")
                .arg("set-current-epoch")
                .arg(epoch.to_string()),
            false,
        );
    }

    pub fn set_current_account(&mut self, name: &str) {
        let real_name = String::from(name).to_lowercase();
        let account = self
            .accounts
            .get(&real_name)
            .expect("Given account does not exist");
        run_command(
            Command::new("resim")
                .arg("set-default-account")
                .arg(account.address())
                .arg(account.private_key()),
            false,
        );

        self.current_account = real_name;
    }

    pub fn get_current_account(&self) -> &Account {
        self.accounts.get(&self.current_account).unwrap()
    }

    pub fn get_token(&self, name: &str) -> &String {
        self.resource_manager.get_address(name)
    }

    pub fn get_account(&self, name: &str) -> Option<&Account> {
        self.accounts.get(name)
    }

    pub fn amount_owned_by(&self, account: &str, token: &str) -> Decimal {
        match self.accounts.get(account) {
            None => {
                panic!("The account {} does not exist", account)
            }
            Some(acc) =>
                {
                    acc.amount_owned(self.get_token(token))
                }
        }
    }

    pub fn amount_owned_by_current(&self, token: &str) -> Decimal {
       self.get_current_account().amount_owned(self.get_token(token))
    }

    pub fn get_component(&self, name: &str) -> Option<&str> {
        match self.components.get(name) {
            None => None,
            Some(comp) => Some(comp.address()),
        }
    }

    pub fn get_non_fungible_ids_for(&self, account: &str, resource: &str) -> Option<&Vec<String>>
    {
        match self.accounts.get(account) {
            None => {
                panic!("The account {} does not exist", account)
            }
            Some(acc) =>
                {
                    acc.get_non_fungibles_ids(self.resource_manager.get_address(resource))
                }
        }
    }

    pub fn get_non_fungible_ids_for_current(&self, resource: &str) -> Option<&Vec<String>>
    {
        self.get_current_account().get_non_fungibles_ids(self.resource_manager.get_address(resource))
    }


    fn create_manifest<M>(path: &str, method: &M)
        where
            M: Method
    {
        let mut manifest = Manifest::new();
        manifest.call_method(method);
        let manifest_string = manifest.build();
        write_manifest(manifest_string, path, method.name());
    }

    fn get_binding_for(&self, arg: &Arg, arg_count: u32) -> (String, String)
    {
        let arg_name = format!("arg_{}", arg_count);
        let arg_value = match arg
        {
            Arg::Unit => { panic!("Should not happen") }
            Arg::Bool(value) => { format!("{}", *value) }
            Arg::I8(value)  => { format!("{}", *value) }
            Arg::I16(value) => { format!("{}", *value) }
            Arg::I32(value)  => { format!("{}", *value) }
            Arg::I64(value) => { format!("{}", *value) }
            Arg::I128(value)  => { format!("{}", *value) }
            Arg::U8(value)  => { format!("{}", *value) }
            Arg::U16(value)  => { format!("{}", *value) }
            Arg::U32(value)  => { format!("{}", *value) }
            Arg::U64(value)  => { format!("{}", *value) }
            Arg::U128(value) => { format!("{}", *value) }
            Arg::StringArg(value) | Arg::SystemAddressArg(value) | Arg::Expression(value) | Arg::Blob(value) | Arg::HashArg(value) | Arg::EcdsaSecp256k1PublicKeyArg(value) | Arg::EcdsaSecp256k1Signature(value) | Arg::EddsaEd25519PublicKey(value) | Arg::EddsaEd25519Signature(value) => { format!("{}", *value) }

            Arg::EnumArg(variant, fields) =>
                {
                    format!("{}, {}", variant, self.get_binding_for_elements(fields))
                }
            Arg::TupleArg(elements)| Arg::VecArg(elements) =>
                {
                    format!("{}", self.get_binding_for_elements(elements))
                }
            Arg::HashMapArg(elements) =>
                {
                    let mut string = String::new();
                    for (key_arg, value_arg) in elements
                    {
                        let (_, key_value) = self.get_binding_for(key_arg, 0);
                        let (_, value_value) = self.get_binding_for(value_arg, 0);
                        string = format!("{}Tuple({}, {}), ", string, key_value, value_value);
                    }
                    string.pop();
                    string.pop();
                    string
                }
            Arg::PackageAddressArg(name) =>
                {
                    match self.packages.get(name)
                    {
                        None => { panic!("No package with name {}", name) }
                        Some(package) => { String::from(package.address()) }
                    }
                }
            Arg::ComponentAddressArg(name) =>
                {
                    match self.get_component(&name)
                    {
                        None => { panic!("No components with name {}", name) }
                        Some(comp) => { String::from(comp) }
                    }
                }
            Arg::AccountAddressArg(name) =>
                {
                    match self.get_account(&name)
                    {
                        None => { panic!("No account with name {}", name) }
                        Some(account) => { String::from(account.address()) }
                    }
                }

            Arg::ResourceAddressArg(name) =>
                {
                    self.resource_manager.get_address(name).clone()
                }
            Arg::DecimalArg(value)=>
                {
                    value.to_string()
                }
            Arg::PreciseDecimalArg(value) =>
                {
                    value.to_string()
                }
                Arg::NonFungibleIdArg(arg) =>
                {
                   let (_, value) =  self.get_binding_for(arg.as_ref(), arg_count);
                    value
                }
            Arg::FungibleBucketArg(_, _)| Arg::NonFungibleBucketArg(_, _) | Arg::FungibleProofArg(_,_) | Arg::NonFungibleProofArg(_,_) => { panic!("This should not happen") }
            Arg::NonFungibleAddressArg(name, arg) =>
                {
                    let (_, id_value) = self.get_binding_for(arg.as_ref(), 0);
                    let resource_value = self.resource_manager.get_address(name);

                    format!("{}, {}", resource_value, id_value)
                }
        };

        (arg_name, arg_value)
    }

    fn get_binding_for_elements(&self, args: &Vec<Arg>) -> String
    {
        let mut string = String::new();
        for arg in args
        {
            let (_, value) = self.get_binding_for(arg, 0);
            string = format!("{}{}, ", string, value)
        }
        string.pop();
        string.pop();
        string
    }
}