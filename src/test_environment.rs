use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;
use lazy_static::lazy_static;
use crate::account::Account;
use crate::component::Component;
use crate::package::Package;
use regex::Regex;
use scrypto::{dec};
use scrypto::math::Decimal;
use scrypto::prelude::{ComponentAddress};
use crate::manifest::Manifest;
use crate::method::{Method};
use crate::utils::{run_command, run_manifest};
use crate::RADIX_TOKEN;

pub struct TestEnvironment
{
    accounts: HashMap<String, Account>,
    packages: HashMap<String, Package>,
    components: HashMap<String, Component>,
    current_account: String,
    tokens: HashMap<String, String>
}

impl TestEnvironment
{
    pub fn new() -> TestEnvironment
    {
        Self::reset();

        let default_account = Account::new();
        let mut accounts = HashMap::new();
        accounts.insert(String::from("default"), default_account);
        let mut tokens = HashMap::new();
        tokens.insert(String::from("radix"), String::from(RADIX_TOKEN));
       TestEnvironment
       {
           accounts,
           packages: HashMap::new(),
           components: HashMap::new(),
           current_account: String::from("default"),
           tokens
       }
    }

    pub fn create_account(&mut self, name: &str)
    {
        let real_name = String::from(name).to_lowercase();
        if self.accounts.contains_key(&real_name)
        {
            panic!("An account with this name already exists");
        }
        else
        {
            self.accounts.insert(String::from(real_name), Account::new());
        }
    }

    pub fn create_fixed_supply_token(&mut self, name: &str, initial_supply: Decimal)
    {
        let real_name = String::from(name).to_lowercase();
        match self.tokens.get(&real_name)
        {
            Some(_) => { panic!("A token with same name already exists!") }
            None =>
                {
                    let output = run_command(Command::new("resim")
                        .arg("new-token-fixed")
                        .arg(initial_supply.to_string()));

                    lazy_static!{
                        static ref ADDRESS_RE: Regex = Regex::new(r#"ResourceAddress("(\w*)")"#).unwrap();
                    }

                    let resource_address = String::from(&ADDRESS_RE.captures(&output).unwrap()[1]);

                    self.tokens.insert(real_name, resource_address);
                }
        }
    }

    pub fn publish_package(&mut self, name: &str, mut package: Package, path: &str)
    {
        let real_name = String::from(name).to_lowercase();

        if !self.packages.contains_key(&real_name)
        {
            lazy_static! {
            static ref PACKAGE_RE: Regex = Regex::new(r"Success! New Package: (\w*)").unwrap();
            }

            let package_output = run_command(Command::new("resim")
                .arg("publish")
                .arg(path));

            let package_address = &PACKAGE_RE.captures(&package_output)
                .expect(&format!("Something went wrong! Maybe the path was incorrect? \n{}", package_output))[1];

            package.set_address(String::from(package_address));
            self.packages.insert(real_name, package);
        }
        else
        {
            panic!("A package with the same name already exists!");
        }

    }

    pub fn new_component(&mut self, name: &str, package_name: &str, blueprint_name: &str)
    {
        if self.components.contains_key(name)
        {
            panic!("A component with the same name already exists!")
        }

        match self.packages.get(package_name)
        {
            Some(package) =>
                {
                    match package.get_blueprint(blueprint_name)
                    {
                        Some(box_blueprint) =>
                            {
                                let blueprint = box_blueprint.as_ref();
                                let (inst_name, args) = blueprint.instantiate();

                                let output = run_command(Command::new("resim")
                                    .arg("call-function")
                                    .arg(package.address())
                                    .arg(blueprint.name())
                                    .arg(inst_name)
                                    .args(args));

                                lazy_static! {
                                    static ref COMPONENT_RE: Regex = Regex::new(r#"ComponentAddress\("(\w*)"\)"#).unwrap();
                                }

                                let component_address = &COMPONENT_RE.captures(&output)
                                    .expect(&format!("Something went wrong when trying to instantiate blueprint! \n{}", output))[1];

                                let comp = Component::from(component_address);
                                self.update_tokens_from_comp(&comp);
                                self.components.insert(String::from(name), comp);

                            }
                        None =>
                            { panic!("Could not find a blueprint named {} for the package {}", blueprint_name, package_name); }
                    }
                }
            None => { panic!("Could not find a package named {}", name); }
        }
    }

    pub fn call_method<M>(&mut self, component: &str, method: M)
        where M: Method
    {
        let account_comp = ComponentAddress::from_str(self.get_current_account().address())
            .expect("Fatal Error: The stored address of the current account is faulty!");

        match self.components.get_mut(component)
        {
            None => { panic!("No component with name {}", component) }
            Some(comp) =>
                {
                    let component_address = ComponentAddress::from_str(comp.address())
                        .expect("Fatal Error: The stored address of the given component is faulty!");

                    let mut manifest = Manifest::new();
                    manifest.lock_fee( account_comp.clone(), dec!(100));
                    let method_name = method.name();
                    manifest.call_method(&method, component_address, account_comp.clone(), &self.tokens);
                    manifest.deposit_batch(account_comp);

                    run_manifest(manifest, method_name);

                    comp.update_resources();
                    self.update_current_account();
                }
        }
    }

    fn update_tokens_from_comp(&mut self, component : &Component)
    {
        let output = run_command(Command::new("resim")
            .arg("show")
            .arg(component.address()));

        lazy_static! {
            static ref RESOURCES_RE: Regex = Regex::new(r#".─ \{ amount: ([\d.]*), resource address: (\w*), name: "(\w*)", "#).unwrap();
        }
        for resource in RESOURCES_RE.captures_iter(&output)
        {
            let address = &resource[2];
            let name = &resource[3];
            self.try_add_token(name, address);
        }
    }

    pub fn reset()
    {
        run_command(Command::new("resim").arg("reset"));
    }

    fn update_current_account(&mut self)
    {
        self.accounts.get_mut(&self.current_account).unwrap().update_resources();
    }

    fn try_add_token(&mut self, name: &str, address: &str)
    {
        let real_name = String::from(name).to_lowercase();
        match self.tokens.get(&real_name)
        {
            Some(_) => {},
            None =>
                {
                    self.tokens.insert(real_name, String::from(address));
                }
        }
    }

    pub fn set_current_epoch(&mut self, epoch: u64)
    {
        run_command(Command::new("resim")
            .arg("set-current-epoch")
            .arg(epoch.to_string()));
    }

    pub fn set_current_account(&mut self, name: &str)
    {
        let real_name = String::from(name).to_lowercase();
        run_command(Command::new("resim")
            .arg("set-default-account")
            .arg(self.accounts.get(&real_name).expect("Given account does not exist").address()));

        self.current_account = real_name;
    }

    pub fn get_current_account(&self) -> &Account
    {
        self.accounts.get(&self.current_account).unwrap()
    }

    pub fn get_token(&self, name: &str) -> Option<&String>
    {
        let real_name = String::from(name).to_lowercase();
        self.tokens.get(&real_name)
    }
}
