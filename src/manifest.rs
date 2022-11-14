use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use scrypto::math::Decimal;
use scrypto::prelude::{ComponentAddress, ResourceAddress};
use crate::instructions::Instruction;
use crate::method::{Arg, Method};

pub struct Manifest{
    needed_resources: Vec<Instruction>,
    instructions: Vec<Instruction>,
    id: u32,
    has_proofs: bool
}

impl Manifest
{
    pub fn new() -> Self{
        Self{
            needed_resources: vec![],
            instructions: Vec::new(),
            id: 0,
            has_proofs: false
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
                        args_vec.push(self.deal_with_arg(&arg, &caller_address, &tokens));
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

    pub fn create_proof(&mut self, caller_address: ComponentAddress, resource_address: ResourceAddress, proof_id : u32) -> &mut Self
    {
        let inst_1 = Instruction::CallMethod {
            component_address: caller_address,
            method_name: "create_proof".to_string(),
            args: vec![format!("ResourceAddress(\"{}\")", resource_address)]
        };

        let inst_2 = Instruction::CreateProofFromAuthZone {
            resource_address,
            proof_id
        };

        self.needed_resources.push(inst_1);
        self.needed_resources.push(inst_2);

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

    pub fn drop_proofs(&mut self) -> &mut Self
    {
        if self.has_proofs
        {
            let inst = Instruction::DropAllProofs;
            self.instructions.push(inst);
        }

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

    fn deal_with_arg(&mut self, arg: &Arg, caller_address: &ComponentAddress, tokens: &HashMap<String,String>) -> String
    {
        match arg
        {
            Arg::Unit => { format!("()") }
            Arg::Bool(b) => { format!("{}", b) }
            Arg::I8(int) => { format!("{}i8", int) }
            Arg::I16(int) => { format!("{}i16", int) }
            Arg::I32(int) => { format!("{}i32", int) }
            Arg::I64(int) => { format!("{}i64", int) }
            Arg::I128(int) => { format!("{}i128", int) }
            Arg::U8(uint) => { format!("{}u8", uint) }
            Arg::U16(uint) => { format!("{}u16", uint) }
            Arg::U32(uint) => { format!("{}u32", uint) }
            Arg::U64(uint) => { format!("{}u64", uint) }
            Arg::U128(uint) => { format!("{}u128", uint) }
            Arg::StringArg(str) => { format!("\"{}\"", str) }
            Arg::Struct(_, fields) => { format!("Struct({})", self.deal_with_arg_vec(fields, caller_address, tokens)) }
            Arg::OptionArg(_, opt) =>
                {
                    match opt
                    {
                        None => { format!("None") }
                        Some(arg) => { format!("Some({})", self.deal_with_arg(arg, caller_address, tokens)) }
                    }
                }
            Arg::BoxArg(arg) => { format!("Box({})", self.deal_with_arg(arg, caller_address, tokens)) }
            Arg::TupleArg(elements) => { format!("Tuple({})",  self.deal_with_arg_vec(elements, caller_address, tokens)) }
            Arg::ResultArg(_,_, res) =>
                {
                    match (*res).deref()
                    {
                        Ok(arg) => { format!("Ok({})", self.deal_with_arg(arg, caller_address, tokens)) }
                        Err(arg) => { format!("Err({})",self.deal_with_arg(arg, caller_address, tokens)) }
                    }
                }
            Arg::VecArg(elements) =>
                {
                    let el_type = match elements.first()
                    {
                        None => { String::from("()") }
                        Some(arg) => { arg.get_type() }
                    };

                    format!("Vec<{}>({})", el_type,  self.deal_with_arg_vec(elements, caller_address, tokens))
                }
            Arg::HashMapArg(_,_, map) =>
                {
                    let elements = Vec::from_iter(map.iter());
                    let el_type = match elements.first()
                    {
                        None => { String::from("(),()") }
                        Some((key_arg, value_arg)) =>
                            { format!("{},{}", key_arg.get_type(), value_arg.get_type()) }
                    };

                    let mut vec_str: Vec<Arg> = vec![];
                    for element in elements
                    {
                        vec_str.push(element.0.clone());
                        vec_str.push(element.1.clone());
                    }
                    format!("Map<{}>({})", el_type, self.deal_with_arg_vec(&vec_str, caller_address, tokens))
                }
            Arg::DecimalArg(dec) => { format!("Decimal(\"{}\")", dec) }
            Arg::PreciseDecimalArg(predec) => { format!("PreciseDecimal(\"{}\")", predec) }
            Arg::PackageAddressArg(str) => { format!("PackageAddress(\"{}\")", str) }
            Arg::ComponentAddressArg(str) => { format!("ComponentAddress(\"{}\")", str) }
            Arg::ResourceAddressArg(str) => { format!("ResourceAddress(\"{}\")", str) }
            Arg::NonFungibleAddressArg(str) => { format!("NonFungibleAddress(\"{}\")", str) }
            Arg::HashArg(str) => { format!("Hash(\"{}\")", str) }
            Arg::BucketArg(name, amount) =>
                {
                    let token_str = tokens.get(&name.to_lowercase())
                        .expect(&format!("Could not find token {} in the list of tokens", name));
                    let token_address = ResourceAddress::from_str(token_str)
                        .expect("Error! The recorder address of the token is faulty!");

                    self.withdraw_by_amount(caller_address.clone(), amount.clone(), token_address.clone());
                    self.take_from_worktop_by_amount(amount.clone(), token_address, self.id);
                    let ret = format!("Bucket(\"{}\")", self.id);
                    self.id+=1;
                    ret

                }
            Arg::ProofArg(name) =>
                {
                    let token_str = tokens.get(&name.to_lowercase())
                        .expect(&format!("Could not find token {} in the list of tokens", name));
                    let token_address = ResourceAddress::from_str(token_str)
                        .expect("Error! The recorder address of the token is faulty!");

                    self.create_proof(caller_address.clone(), token_address, self.id);
                    let ret = format!("Proof(\"{}\")", self.id);
                    self.id+=1;
                    self.has_proofs = true;

                    ret

                }
            Arg::NonFungibleIdArg(name) =>
                {
                    format!("NonFungibleId(\"{}\")", name)
                }
        }
    }

    fn deal_with_arg_vec(&mut self, vec: &Vec<Arg>, caller_address: &ComponentAddress, tokens: &HashMap<String,String>) -> String
    {
        let mut vec_str = String::new();
        for element in vec
        {
            vec_str = format!("{}{}, ", vec_str, self.deal_with_arg(element, caller_address, tokens));
        }
        vec_str.pop();
        vec_str.pop();
        vec_str
    }
}