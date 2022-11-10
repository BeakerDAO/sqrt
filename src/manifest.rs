use std::collections::HashMap;
use std::str::FromStr;
use scrypto::math::Decimal;
use scrypto::prelude::{ComponentAddress, ResourceAddress};
use crate::instructions::Instruction;
use crate::method::{Arg, Method};

pub struct Manifest{
    needed_resources: Vec<Instruction>,
    instructions: Vec<Instruction>,
    bucket_id: u32,
}

impl Manifest
{
    pub fn new() -> Self{
        Self{
            needed_resources: vec![],
            instructions: Vec::new(),
            bucket_id: 0,
        }
    }

    pub fn call_method<M>(&mut self, method: &M , component_address: ComponentAddress, caller_address: ComponentAddress, tokens: &HashMap<String,String>) -> &mut Self
    where M: Method
    {
        let mut args_vec: Vec<String> = Vec::new();

        match method.args()
        {
            None => {}
            Some(args) =>
                {
                    for arg in args
                    {
                        match arg
                        {
                            Arg::Bucket(name, amount) =>
                                {
                                    let token_str = tokens.get(&name.to_lowercase())
                                        .expect(&format!("Could not find token {} in the list of tokens", name));
                                    let token_address = ResourceAddress::from_str(token_str)
                                        .expect("Error! The recorder address of the token is faulty!");

                                    self.withdraw_by_amount(caller_address.clone(), amount.clone(), token_address.clone());
                                    self.take_from_worktop_by_amount(amount, token_address, self.bucket_id);
                                    let arg_str = format!("Bucket(\"{}\")", self.bucket_id);

                                    args_vec.push(arg_str);
                                    self.bucket_id+=1;

                                }
                            Arg::Other(str) =>
                                {
                                    args_vec.push(str);
                                }
                        }
                    }
                }
        }

        let inst = Instruction::CallMethod{
            component_address,
            method_name: method.name().to_string(),
            args: args_vec
        };

        self.instructions.push(inst);
        self
    }


    pub fn lock_fee(&mut self, caller_address: ComponentAddress, amount: Decimal) -> &mut Self
    {
        let inst = Instruction::CallMethod {
            component_address: caller_address,
            method_name: "lock_fee".to_string(),
            args: vec![format!("Decimal(\"{}\")", amount)]
        };
        self.needed_resources.push(inst);

        self
    }

    fn take_from_worktop_by_amount(&mut self, amount: Decimal, resource_address: ResourceAddress, bucket_id: u32) -> &mut Self
    {
        let inst = Instruction::TakeFromWorktopByAmount {
            amount,
            resource_address,
            bucket_id
        };
        self.needed_resources.push(inst);

        self
    }

    fn withdraw_by_amount(&mut self, caller_address: ComponentAddress, amount: Decimal, resource_address: ResourceAddress) -> &mut Self
    {
        let inst = Instruction::CallMethod {
            component_address: caller_address,
            method_name: "withdraw_by_amount".to_string(),
            args: vec![format!("Decimal(\"{}\")", amount), format!("ResourceAddress(\"{}\")", resource_address)]
        };
        self.needed_resources.push(inst);

        self
    }

    pub fn deposit_batch(&mut self, caller_address: ComponentAddress) -> &mut Self
    {
        let inst = Instruction::CallMethod
        {
            component_address: caller_address,
            method_name: "deposit_batch".to_string(),
            args: vec![String::from("Expression(\"ENTIRE_WORKTOP\")")]
        };

        self.instructions.push(inst);

        self

    }

    pub fn build(&self) -> String
    {
        let mut output = String::new();
        for instr in &self.needed_resources
        {
            output = format!("{}\n\n{}", output, instr);
        }
        for instr in &self.instructions
        {
            output = format!("{}\n\n{}", output, instr);
        }

        output
    }
}