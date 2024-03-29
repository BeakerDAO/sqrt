use crate::utils::{generate_owner_badge, run_command};
use lazy_static::lazy_static;
use regex::Regex;
use scrypto::prelude::Decimal;
use std::collections::HashMap;
use std::process::Command;

pub struct Account {
    address: String,
    private_key: String,
    owner_badge: String,
    fungibles: HashMap<String, Decimal>,
    non_fungibles: HashMap<String, Vec<String>>,
}

impl Account {
    pub fn new() -> Account {
        let account_command = run_command(Command::new("resim").arg("new-account"), false);
        let badge = generate_owner_badge();
        Self::from(&account_command.0, badge)
    }

    pub fn from(string_with_info: &str, badge_address: String) -> Account {
        lazy_static! {
            static ref ADDRESS_RE: Regex = Regex::new(r"Account component address: (\w*)").unwrap();
            static ref PRIVATE_KEY_RE: Regex = Regex::new(r"Private key: (\w*)").unwrap();
        }

        let address = &ADDRESS_RE
            .captures(string_with_info)
            .expect("Could not find address from given string")[1];
        let private_key = &PRIVATE_KEY_RE
            .captures(string_with_info)
            .expect("Could not find private key from given string")[1];

        Account {
            address: String::from(address),
            private_key: String::from(private_key),
            owner_badge: badge_address,
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn owner_badge(&self) -> &str {
        &self.owner_badge
    }
    pub fn private_key(&self) -> &str {
        &self.private_key
    }

    pub fn amount_owned(&self, resource: &String) -> Decimal {
        match self.non_fungibles.get(resource) {
            None => {}
            Some(ids) => return Decimal::from(ids.len()),
        }

        match self.fungibles.get(resource) {
            None => Decimal::zero(),
            Some(amount) => *amount,
        }
    }

    pub fn get_non_fungibles_ids(&self, address: &String) -> Option<&Vec<String>> {
        self.non_fungibles.get(address)
    }

    pub fn update_fungible(&mut self, address: &String, new_amount: Decimal) {
        match self.fungibles.get_mut(address) {
            None => {
                self.fungibles.insert(address.clone(), new_amount);
            }
            Some(amount) => {
                *amount = new_amount;
            }
        }
    }

    pub fn update_non_fungibles(&mut self, address: &String, new_ids: Vec<String>) {
        match self.non_fungibles.get_mut(address) {
            None => {
                self.non_fungibles.insert(address.clone(), new_ids);
            }
            Some(ids) => {
                *ids = new_ids;
            }
        }
    }
}
