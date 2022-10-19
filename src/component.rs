use std::collections::HashMap;
use std::process::Command;
use lazy_static::lazy_static;
use scrypto::math::Decimal;
use regex::Regex;
use crate::utils::run_command;

pub struct Component
{
    address: String,
    package_address: String,
    blueprint_name: String,
    resources: HashMap<String, Decimal>,
}

impl Component
{
    pub fn from(address: &str, package_address: &str, blueprint_name: &str) -> Component
    {
        let mut comp = Component
        {
            address: String::from(address),
            package_address: String::from(package_address),
            blueprint_name: String::from(blueprint_name),
            resources: HashMap::new()
        };
        comp.update_resources();
        comp
    }

    pub fn update_resources(&mut self)
    {
        let info = run_command(Command::new("resim")
            .arg("show")
            .arg(&self.address));

        // Resource line is of the form
        // amount: 1000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD"
        lazy_static! {
            static ref RESOURCE_RE: Regex = Regex::new(r#".─ \{ amount: ([\d.]*), resource address: (\w*),"#).unwrap();
        }

        for cap in RESOURCE_RE.captures_iter(&info)
        {
            let amount = Decimal::from(&cap[1]);
            let address = &cap[2];
            self.update_resource(address, amount);
        }
    }

    pub fn update_resource(&mut self, resource_address: &str, new_amount: Decimal)
    {
        match self.resources.get_mut(resource_address)
        {
            None => { self.resources.insert(String::from(resource_address), new_amount); }
            Some(amount) => { *amount = new_amount; }
        }

    }


    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn package_address(&self) -> &str {
        &self.package_address
    }
}