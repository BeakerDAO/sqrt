use std::collections::HashMap;
use std::process::Command;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use scrypto::prelude::Decimal;
use crate::account::Account;
use crate::utils::run_command;

pub struct ResourceManager {
    resources: HashMap<String, String>,
    is_fungible: HashMap<String, bool>
}


impl ResourceManager {

    pub fn new() -> ResourceManager {
        ResourceManager {
            resources: HashMap::new(),
            is_fungible: HashMap::new()
        }
    }

    pub fn update_resources(&mut self)
    {
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

            lazy_static! {
                static ref FUNGIBLE_RE: Regex = Regex::new(r#"ResourceType: Fungible"#).unwrap();
            }

            match &NAME_RE.captures(&output_show) {
                None => {}
                Some(name) => {
                    let is_fungible = FUNGIBLE_RE.is_match(&output_show);
                    self.add_resource(&String::from(&name[1]), final_address, is_fungible)
                }
            }
        }
    }

    pub fn update_resources_for_account(&self, account: &mut Account)
    {
        let account_resources = run_command(Command::new("resim").arg("show").arg(account.address()), false);

        lazy_static! {
            static ref RESOURCE_RE: Regex = Regex::new(r#"\{ amount: ([\d.]*), resource address: (\w*) \}"#).unwrap();
        }

        lazy_static! {
            static ref NON_FUNGIBLE_RE: Regex = Regex::new(r#"NonFungible { id: NonFungibleId\( (.*) \), immutable_data"#).unwrap();
        }

        let mut non_fungible_vec: Vec<Captures> = NON_FUNGIBLE_RE.captures_iter(&account_resources).collect();

        for resource in RESOURCE_RE.captures_iter(&account_resources)
        {
            let amount = Decimal::from(&resource[1]);
            let address = String::from(&resource[2]);
            if self.is_fungible(&address)
            {
                account.update_fungible(&address, amount);
            }
            else
            {
                let amount_int: i32 = amount.0.try_into().expect("Non integer amount of non fungible resources is impossible");
                let mut ids = vec![];
                for _ in 0..amount_int
                {
                    let nf_resource = non_fungible_vec.remove(0);
                    let nf_id = &nf_resource[1];
                    ids.push(String::from(nf_id));
                }

                account.update_non_fungibles(&address, ids);
            }
        }
    }

    pub fn exists(&self, name: &String) -> bool
    {
        self.resources.contains_key(name)
    }

    pub fn add_resource(&mut self, name: &String, resource_address: String, is_fungible: bool)
    {
        let recorded_name = Self::recorded_name(name);
        if !self.exists(&recorded_name)
        {
            self.resources.insert(name.clone(), resource_address.clone());
            self.is_fungible.insert(resource_address, is_fungible);
        }
    }

    pub fn get_address(&self, name: &str) -> &String
    {
        let recorded_name = Self::recorded_name(&String::from(name));
        let error = format!("The resource {} does not exist!", name);
        self.resources.get(&recorded_name).expect(&error)
    }

    pub fn is_fungible(&self, address: &String) -> bool
    {
        *self.is_fungible.get(address).unwrap()
    }

    fn recorded_name(name: &String) -> String
    {
        name.to_lowercase()
    }
}