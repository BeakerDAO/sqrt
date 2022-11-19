use std::collections::HashMap;
use std::process::Command;
use scrypto::math::Decimal;
use regex::Regex;
use lazy_static::lazy_static;
use crate::utils::run_command;

pub struct Account{
    address: String,
    public_key: String,
    private_key: String,
    resources: HashMap<String, Decimal>
}

impl Account
{

    pub fn new() -> Account
    {
        let account_command = run_command(Command::new("resim")
            .arg("new-account"),
            false);
        Self::from(&account_command)
    }

    pub fn from(string_with_info: &str) -> Account
    {
        lazy_static! {
            static ref ADDRESS_RE: Regex = Regex::new(r"Account component address: (\w*)").unwrap();
            static ref PUBLIC_KEY_RE:  Regex = Regex::new(r"Public key: (\w*)").unwrap();
            static ref PRIVATE_KEY_RE: Regex = Regex::new(r"Private key: (\w*)").unwrap();
        }

        let address = &ADDRESS_RE.captures(string_with_info).expect("Could not find address from given string")[1];
        let public_key = &PUBLIC_KEY_RE.captures(string_with_info).expect("Could not find public key from given string")[1];
        let private_key = &PRIVATE_KEY_RE.captures(string_with_info).expect("Could not find private key from given string")[1];

        Account{
            address: String::from(address),
            public_key: String::from(public_key),
            private_key: String::from(private_key),
            resources: HashMap::new()
        }
    }

    pub fn update_resources_from(&mut self, string_with_info: &str)
    {
        // Resource line is of the form
        // amount: 1000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD"

        lazy_static! {
            static ref RESOURCE_RE: Regex = Regex::new(r#".─ \{ amount: ([\d.]*), resource address: (\w*),"#).unwrap();
        }
        for cap in RESOURCE_RE.captures_iter(string_with_info)
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

    pub fn update_resources(&mut self)
    {
        let info = run_command(Command::new("resim")
            .arg("show")
            .arg(&self.address),
            false);

        // Resource line is of the form
        // amount: 1000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD"
        lazy_static! {
            static ref RESOURCE_RE: Regex = Regex::new(r#"amount: ([\d.]*), resource address: (\w*)"#).unwrap();
        }

        for cap in RESOURCE_RE.captures_iter(&info)
        {
            let amount = Decimal::from(&cap[1]);
            let address = &cap[2];
            self.update_resource(address, amount);
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

    pub fn amount_owned(&self, resource: &String) -> Decimal
    {
        match self.resources.get(resource)
        {
            None => { Decimal::zero() }
            Some(amount) => { *amount }
        }
    }

}


#[cfg(test)]
mod tests
{
    use std::collections::HashMap;
    use scrypto::dec;
    use crate::account::Account;

    #[test]
    fn test_from()
    {
        let resim_output = "A new account has been created!
        Account component address: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58
        Public key: 02352a538f5be9d2312f8f3e7ec0a7886e5e438dab401b7b144f790812b25f7abc
        Private key: b37a8339777e8cf1e69cb77010f2e82a47bffec9c6ea0ca49d796f367bd0cb3f";

        let account = Account::from(resim_output);
        assert_eq!(account.address,"account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58");
        assert_eq!(account.public_key, "02352a538f5be9d2312f8f3e7ec0a7886e5e438dab401b7b144f790812b25f7abc");
        assert_eq!(account.private_key, "b37a8339777e8cf1e69cb77010f2e82a47bffec9c6ea0ca49d796f367bd0cb3f");
    }

    #[test]
    fn test_update_assets_from_single()
    {
        let resim_output = "Component: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58
        Blueprint: { package_address: package_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpsuluv44, blueprint_name: \"Account\" }
        Authorization
        ├─ \"deposit_batch\" => AllowAll
        ├─ \"balance\" => AllowAll
        └─ \"deposit\" => AllowAll
        State: Struct(KeyValueStore(\"6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc502040000\"))
        Key Value Store: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58(6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc5, 1026)
        └─ ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\") => Vault(\"6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc504040000\")
        Resources:
        └─ { amount: 1000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: \"Radix\", symbol: \"XRD\" }";

        let mut account = Account{
            address: "".to_string(),
            public_key: "".to_string(),
            private_key: "".to_string(),
            resources: HashMap::new()
        };

        account.update_resources_from(resim_output);
        assert_eq!(*account.resources.get("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag").unwrap(), dec!(1000));
    }

    #[test]
    fn test_update_assets_from_multiple()
    {
        let resim_output = "Component: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58
        Blueprint: { package_address: package_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpsuluv44, blueprint_name: \"Account\" }
        Authorization
        ├─ \"deposit_batch\" => AllowAll
        ├─ \"balance\" => AllowAll
        └─ \"deposit\" => AllowAll
        State: Struct(KeyValueStore(\"6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc502040000\"))
        Key Value Store: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58(6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc5, 1026)
        └─ ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\") => Vault(\"6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc504040000\")
        Resources:
        └─ { amount: 1000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: \"Radix\", symbol: \"XRD\" }
        └─ { amount: 123, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57pol, name: \"Test\", symbol: \"TST\" }";

        let mut account = Account{
            address: "".to_string(),
            public_key: "".to_string(),
            private_key: "".to_string(),
            resources: HashMap::new()
        };

        account.update_resources_from(resim_output);
        assert_eq!(*account.resources.get("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag").unwrap(), dec!(1000));
        assert_eq!(*account.resources.get("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57pol").unwrap(), dec!(123));
    }

    #[test]
    fn test_update_assets_from_already_existing()
    {
        let resim_output = "Component: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58
        Blueprint: { package_address: package_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpsuluv44, blueprint_name: \"Account\" }
        Authorization
        ├─ \"deposit_batch\" => AllowAll
        ├─ \"balance\" => AllowAll
        └─ \"deposit\" => AllowAll
        State: Struct(KeyValueStore(\"6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc502040000\"))
        Key Value Store: account_sim1qwyg0hev5qehp67fln4g5t3rmf0pgazs4sylvecl7zzsu3sa58(6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc5, 1026)
        └─ ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\") => Vault(\"6681014ff536f2e75167c6355de07a659657ff09cf9f39524f53cee4d8185dc504040000\")
        Resources:
        └─ { amount: 1000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: \"Radix\", symbol: \"XRD\" }
        └─ { amount: 123, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57pol, name: \"Test\", symbol: \"TST\" }";

        let mut account = Account{
            address: "".to_string(),
            public_key: "".to_string(),
            private_key: "".to_string(),
            resources: HashMap::new()
        };
        account.update_resource("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag", dec!(100));
        assert_eq!(*account.resources.get("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag").unwrap(), dec!(100));

        account.update_resources_from(resim_output);
        assert_eq!(*account.resources.get("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag").unwrap(), dec!(1000))
    }
}