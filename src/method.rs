use std::collections::HashMap;
use std::str::FromStr;
use scrypto::component::ComponentAddress;
use scrypto::engine::types::BucketId;
use scrypto::math::Decimal;
use scrypto::prelude::ResourceAddress;
use scrypto::resource::Bucket;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

pub trait Method
{
    /// Returns the name of the blueprint method
    fn name(&self) -> &str;

    /// Returns the arguments of the blueprint method
    fn args(&self) -> Option<Vec<Args>>;

}

pub enum Args
{
    /// Bucket with resource to send. The String represents the name of the resource and the Decimal the amount to send
    Bucket(String, Decimal),
    Other(String)
}

impl Args
{
    pub fn take_resource<'a>(&'a self, manifest: &'a mut ManifestBuilder, account: ComponentAddress, tokens: &HashMap<String,String>, buckets: &mut Vec<BucketId>) -> &mut ManifestBuilder
    {
        match self
        {
            Args::Bucket(address, amount) =>
                {
                    let token_str = tokens.get(address)
                        .expect(&format!("Could not find token {} in the list of tokens", address));
                    let token_address = ResourceAddress::from_str(token_str).expect("Error! The recorder address of the token is faulty!");
                    let final_manifest = manifest.withdraw_from_account_by_amount(amount.clone(), token_address.clone(), account);
                    let (final_manifest, bucket_id, _) = final_manifest.add_instruction(Instruction::TakeFromWorktopByAmount {
                        amount: amount.clone(),
                        resource_address: token_address,
                    });

                    match bucket_id
                    {
                        None => {}
                        Some(id) => { buckets.push(id); }
                    }

                    final_manifest
                }
            Args::Other(_) => { manifest }
        }
    }

    pub fn add_arg(&self, args: &mut Vec<u8>, buckets: &mut Vec<BucketId>)
    {
        match self
        {
            Args::Bucket(_, _) =>
                {
                    let id = if buckets.is_empty()
                    {
                        panic!("Fatal Error : Not enough bucket ids for function call!");
                    }
                    else
                    {
                        buckets.remove(0)
                    };
                    let mut encoded = scrypto::buffer::scrypto_encode(&Bucket(id));
                    args.append(&mut encoded);
                }
            Args::Other(str) =>
                {
                    let mut encoded = scrypto::buffer::scrypto_encode(str);
                    args.append(&mut encoded);
                }
        }
    }
}