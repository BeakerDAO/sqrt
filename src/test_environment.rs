use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;
use lazy_static::lazy_static;
use crate::account::Account;
use crate::component::Component;
use crate::package::Package;
use regex::Regex;
use scrypto::core::NetworkDefinition;
use scrypto::{args, dec};
use scrypto::engine::types::BucketId;
use scrypto::prelude::ComponentAddress;
use crate::method::{Method};
use crate::utils::run_command;
use transaction::builder::ManifestBuilder;
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
        if self.accounts.contains_key(name)
        {
            panic!("An account with this name already exists");
        }
        else
        {
            self.accounts.insert(String::from(name), Account::new());
        }
    }

    pub fn publish_package(&mut self, name: &str, mut package: Package, path: &str)
    {
        if !self.packages.contains_key(name)
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
            self.packages.insert(String::from(name), package);
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
                                self.components.insert(String::from(name), comp);

                            }
                        None =>
                            { panic!("Could not find a blueprint named {} for the package {}", blueprint_name, package_name); }
                    }
                }
            None => { panic!("Could not find a package named {}", name); }
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
        self.current_account = String::from(name);
        run_command(Command::new("resim")
            .arg("set-default-account")
            .arg(self.accounts.get(name).expect("Given account does not exist").address()));

    }

    pub fn reset()
    {
        run_command(Command::new("resim").arg("reset"));
    }


    pub fn call_method<M>(&mut self, component: &str, method: M)
        where M: Method
    {
        match self.components.get_mut(component)
        {
            None => { panic!("No component with name {}", component) }
            Some(comp) =>
                {
                    let mut buckets: Vec<BucketId> = vec![];
                    let account_comp = ComponentAddress::from_str(&self.accounts.get(&self.current_account).unwrap().address())
                        .expect("Fatal Error: The stored address of the current account is faulty!");

                    let mut manifest = ManifestBuilder::new(&NetworkDefinition::simulator());
                    manifest.lock_fee(dec!(100), account_comp.clone());


                    let pos_args = method.args();
                    let mut method_args: Vec<u8>;
                    match pos_args
                    {
                        None =>
                            {
                                method_args = args![];
                            }
                        Some(args) =>
                            {
                                method_args = vec![];
                                for arg in args
                                {
                                    // Add necessary proofs and resources to worktop and create buckets
                                    arg.take_resource(&mut manifest, account_comp.clone(), &self.tokens, &mut buckets);
                                    // Add encoded args in the args list
                                    arg.add_arg(&mut method_args, &mut buckets);
                                }
                            }
                    }
                    let comp_address = ComponentAddress::from_str(comp.address())
                        .expect("Fatal Error: The stored address of the component is faulty!");
                    manifest.call_method(comp_address, method.name(), method_args);
                    manifest.call_method(account_comp, "deposit_batch", args!("ENTIRE_WORKTOP"));
                    let tr = manifest.build();

                    comp.update_resources();
                    self.accounts.get_mut(&self.current_account).unwrap().update_resources();
                }
        }
    }

}
