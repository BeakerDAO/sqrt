use crate::account::Account;
use crate::component::Component;
use crate::manifest::Manifest;
use crate::method::Method;
use crate::package::Package;
use crate::utils::{create_dir, run_command, run_manifest, transfer, write_transfer};
use crate::RADIX_TOKEN;
use lazy_static::lazy_static;
use regex::Regex;
use scrypto::dec;
use scrypto::math::Decimal;
use scrypto::prelude::{ComponentAddress, ResourceAddress};
use std::collections::HashMap;
use std::process::Command;
use std::str::FromStr;

pub struct TestEnvironment {
    accounts: HashMap<String, Account>,
    packages: HashMap<String, Package>,
    components: HashMap<String, Component>,
    current_account: String,
    tokens: HashMap<String, String>,
}

impl TestEnvironment {
    pub fn new() -> TestEnvironment {
        Self::reset();

        let default_account = Account::new();
        let mut accounts = HashMap::new();
        accounts.insert(String::from("default"), default_account);
        let mut tokens = HashMap::new();
        tokens.insert(String::from("radix"), String::from(RADIX_TOKEN));
        TestEnvironment {
            accounts,
            packages: HashMap::new(),
            components: HashMap::new(),
            current_account: String::from("default"),
            tokens,
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

    pub fn create_fixed_supply_token(&mut self, name: &str, initial_supply: Decimal) -> String {
        let real_name = String::from(name).to_lowercase();
        match self.tokens.get(&real_name) {
            Some(_) => {
                panic!("A token with same name already exists!")
            }
            None => {
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

                self.tokens.insert(real_name, resource_address.clone());
                self.update_current_account();
                resource_address
            }
        }
    }

    pub fn publish_package(&mut self, name: &str, mut package: Package) {
        let real_name = String::from(name).to_lowercase();

        if !self.packages.contains_key(&real_name) {
            lazy_static! {
                static ref PACKAGE_RE: Regex = Regex::new(r"Success! New Package: (\w*)").unwrap();
            }

            let package_output = run_command(
                Command::new("resim").arg("publish").arg(package.path()),
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
                        static ref COMPONENT_RE: Regex =
                            Regex::new(r#"ComponentAddress\("(\w*)"\)"#).unwrap();
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
                    self.update_current_account();
                    self.update_tokens();
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
        let account_comp = ComponentAddress::from_str(self.get_current_account().address())
            .expect("Fatal Error: The stored address of the current account is faulty!");

        let output;
        match self.components.get_mut(component) {
            None => {
                panic!("No component with name {}", component)
            }
            Some(comp) => {
                let component_address = ComponentAddress::from_str(comp.address())
                    .expect("Fatal Error: The stored address of the given component is faulty!");

                let mut manifest = Manifest::new();
                manifest.lock_fee(account_comp.clone(), dec!(100));

                if method.needs_admin_badge() {
                    let raw_address = match comp.admin_badge() {
                        None => {
                            panic!("The component does not have an admin badge!")
                        }
                        Some(str) => str,
                    };

                    let badge_address = ResourceAddress::from_str(raw_address)
                        .expect("Fatal Error: The stored admin badge address is faulty!");

                    manifest.create_proof(account_comp.clone(), badge_address);
                }

                let method_name = method.name();
                manifest.call_method(
                    &method,
                    component_address,
                    account_comp.clone(),
                    &self.tokens,
                );
                manifest.drop_proofs();
                manifest.deposit_batch(account_comp);

                output = run_manifest(manifest, comp.package_path(), method_name);
                self.update_current_account();
                self.update_tokens();
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
            Some(acc) => match self.tokens.get(token) {
                None => {
                    panic!("Token {} does not exist", token)
                }
                Some(tok) => {
                    transfer(
                        &from,
                        acc.address(),
                        tok.as_str(),
                        amount.to_string().as_str(),
                    );
                    acc.update_resources();
                    self.accounts
                        .get_mut(&self.current_account)
                        .unwrap()
                        .update_resources();
                }
            },
        }
    }

    pub fn reset() {
        run_command(Command::new("resim").arg("reset"), false);
    }

    fn update_current_account(&mut self) {
        self.accounts
            .get_mut(&self.current_account)
            .unwrap()
            .update_resources();
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

    pub fn get_token(&self, name: &str) -> Option<&String> {
        let real_name = String::from(name).to_lowercase();
        self.tokens.get(&real_name)
    }

    pub fn get_account(&self, name: &str) -> Option<&Account> {
        self.accounts.get(name)
    }

    pub fn amount_owned_by(&self, account: &str, token: &str) -> Decimal {
        match self.accounts.get(account) {
            None => {
                panic!("The account {} does not exist", account)
            }
            Some(acc) => match self.tokens.get(&token.to_lowercase()) {
                None => {
                    panic!("The token {} does not exist", token)
                }
                Some(tok) => acc.amount_owned(tok),
            },
        }
    }

    pub fn amount_owned_by_current(&self, token: &str) -> Decimal {
        match self.tokens.get(&token.to_lowercase()) {
            None => {
                panic!("The token {} does not exist", token)
            }
            Some(tok) => self.get_current_account().amount_owned(tok),
        }
    }

    pub fn get_component(&self, name: &str) -> Option<&str> {
        match self.components.get(name) {
            None => None,
            Some(comp) => Some(comp.address()),
        }
    }

    fn try_add_token(&mut self, name: &str, address: &str) {
        let real_name = String::from(name).to_lowercase();
        match self.tokens.get(&real_name) {
            Some(_) => {}
            None => {
                self.tokens.insert(real_name, String::from(address));
            }
        }
    }

    fn update_tokens(&mut self) {
        let output = run_command(Command::new("resim").arg("show-ledger"), false);

        lazy_static! {
            static ref RESOURCES_RE: Regex = Regex::new(r#"resource_(\w*)"#).unwrap();
        }

        for resource in RESOURCES_RE.captures_iter(&output) {
            let address = &resource[1];
            let final_address = format!("{}{}", "resource_", address);
            let output_show =
                run_command(Command::new("resim").arg("show").arg(&final_address), false);

            lazy_static! {
                static ref NAME_RE: Regex = Regex::new(r#"name: (.*)"#).unwrap();
            }

            match &NAME_RE.captures(&output_show) {
                None => {}
                Some(name) => {
                    self.try_add_token(&name[1], &final_address);
                }
            }
        }
    }
}
