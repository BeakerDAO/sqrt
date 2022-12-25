use crate::utils::run_command;
use lazy_static::lazy_static;
use regex::Regex;
use scrypto::prelude::Decimal;
use std::collections::HashMap;
use std::process::Command;

pub struct Account {
    address: String,
    public_key: String,
    private_key: String,
    fungibles: HashMap<String, Decimal>,
    non_fungibles: HashMap<String, Vec<String>>
}

impl Account {
    pub fn new() -> Account {
        let account_command = run_command(Command::new("resim").arg("new-account"), false);
        Self::from(&account_command)
    }

    pub fn from(string_with_info: &str) -> Account {
        lazy_static! {
            static ref ADDRESS_RE: Regex = Regex::new(r"Account component address: (\w*)").unwrap();
            static ref PUBLIC_KEY_RE: Regex = Regex::new(r"Public key: (\w*)").unwrap();
            static ref PRIVATE_KEY_RE: Regex = Regex::new(r"Private key: (\w*)").unwrap();
        }

        let address = &ADDRESS_RE
            .captures(string_with_info)
            .expect("Could not find address from given string")[1];
        let public_key = &PUBLIC_KEY_RE
            .captures(string_with_info)
            .expect("Could not find public key from given string")[1];
        let private_key = &PRIVATE_KEY_RE
            .captures(string_with_info)
            .expect("Could not find private key from given string")[1];

        Account {
            address: String::from(address),
            public_key: String::from(public_key),
            private_key: String::from(private_key),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new()
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn private_key(&self) -> &str {
        &self.private_key
    }

    pub fn amount_owned(&self, resource: &String) -> Decimal {
        match self.non_fungibles.get(resource) {
            None => {}
            Some(ids) => { return Decimal::from(ids.len()) }
        }

        match self.fungibles.get(resource) {
            None => Decimal::zero(),
            Some(amount) => *amount,
        }
    }

    pub fn get_non_fungibles_owned(&self, address: &String) -> Option<&Vec<String>> {
        self.non_fungibles.get(address)
    }

    pub fn get_last_non_fungible_id(&self, address: &String) -> Option<&String>
    {
        self.non_fungibles.get(address).unwrap().last()
    }
    pub fn update_fungible(&mut self, address: &String, new_amount: Decimal)
    {
        match self.fungibles.get_mut(address)
        {
            None =>
                {
                    self.fungibles.insert(address.clone(), new_amount);
                }
            Some(amount) =>
                {
                    *amount = new_amount;
                }
        }
    }

    pub fn update_non_fungibles(&mut self, address: &String, new_ids: Vec<String>)
    {
        match self.non_fungibles.get_mut(address)
        {
            None =>
                {
                    self.non_fungibles.insert(address.clone(), new_ids);
                }
            Some(ids) =>
                {
                    *ids = new_ids;
                }
        }
    }
}
