use std::collections::HashMap;
use std::process::Command;
use lazy_static::lazy_static;
use crate::account::Account;
use crate::blueprint::Blueprint;
use crate::component::Component;
use crate::package::Package;
use regex::Regex;
use crate::utils::run_command;

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
        let default_account = Account::new();
        let mut accounts = HashMap::new();
        accounts.insert(String::from("default"), default_account);
       TestEnvironment
       {
           accounts,
           packages: HashMap::new(),
           components: HashMap::new(),
           current_account: String::from("default"),
           tokens: HashMap::new()
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

                                let comp = Component::from(component_address, package.address(), blueprint.name());
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

    pub fn reset(&self)
    {
        run_command(Command::new("resim").arg("reset"));
    }

}