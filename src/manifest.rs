use crate::blueprint::Blueprint;
use crate::instructions::Instruction;
use crate::method::{Arg, Method};
use scrypto::prelude::{dec, Decimal};

pub struct Manifest {
    needed_resources: Vec<Instruction>,
    instructions: Vec<Instruction>,
    id: u32,
    has_proofs: bool,
    arg_count: u32,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            needed_resources: vec![],
            instructions: Vec::new(),
            id: 0,
            has_proofs: false,
            arg_count: 0,
        }
    }

    pub fn instantiate<B>(&mut self, blueprint: &B, args: &Vec<Arg>)
    where
        B: Blueprint + ?Sized,
    {
        self.lock_fee(Self::caller_arg(), dec!(100));
        let args_vec = self.deal_with_args(args);

        let inst = Instruction::CallFunction {
            package_address_arg: Self::package_arg(),
            blueprint_name_arg: blueprint.name().to_string(),
            function_name_arg: blueprint.instantiation_name().to_string(),
            args: args_vec,
        };

        self.instructions.push(inst);
        self.drop_proofs();
        self.deposit_batch(Self::caller_arg());
    }

    pub fn call_method<M>(&mut self, method: &M)
    where
        M: Method,
    {
        self.lock_fee(Self::caller_arg(), dec!(100));
        if method.needs_admin_badge() {
            self.create_admin_badge_proof(Self::caller_arg(), Self::admin_badge_arg());
        }

        let args_vec = match method.args() {
            None => {
                vec![]
            }
            Some(args) => self.deal_with_args(&args),
        };

        let inst = Instruction::CallMethod {
            component_address_arg: Self::component_arg(),
            method_name: method.name().to_string(),
            args: args_vec,
        };

        self.instructions.push(inst);
        self.drop_proofs();
        self.deposit_batch(Self::caller_arg());
    }

    pub fn lock_fee(&mut self, caller_arg: String, amount: Decimal) {
        let inst = Instruction::CallMethod {
            component_address_arg: caller_arg,
            method_name: "lock_fee".to_string(),
            args: vec![format!("Decimal(\"{}\")", amount)],
        };
        self.needed_resources.push(inst);
    }

    fn take_from_worktop_by_amount(
        &mut self,
        amount_arg: String,
        resource_address_arg: String,
        bucket_id: u32,
    ) -> &mut Self {
        let inst = Instruction::TakeFromWorktopByAmount {
            amount_arg,
            resource_address_arg,
            bucket_id,
        };
        self.needed_resources.push(inst);

        self
    }

    fn take_from_worktop_by_ids(
        &mut self,
        resource_address_arg: String,
        ids_arg: String,
        bucket_id: u32,
    ) {
        let inst = Instruction::TakeFromWorktopByIds {
            ids_arg,
            resource_address_arg,
            bucket_id,
        };

        self.needed_resources.push(inst);
    }

    fn withdraw_by_amount(
        &mut self,
        account_arg: String,
        amount_arg: String,
        resource_address_arg: String,
    ) {
        let inst = Instruction::CallMethod {
            component_address_arg: account_arg,
            method_name: "withdraw_by_amount".to_string(),
            args: vec![
                format!("Decimal(\"${{{}}}\")", amount_arg),
                format!("ResourceAddress(\"${{{}}}\")", resource_address_arg),
            ],
        };
        self.needed_resources.push(inst);
    }

    fn withdraw_by_ids(
        &mut self,
        account_arg: String,
        resource_address_arg: String,
        ids_arg: String,
    ) {
        let inst = Instruction::CallMethod {
            component_address_arg: account_arg,
            method_name: "withdraw_by_ids".to_string(),
            args: vec![
                format!("Array<NonFungibleId>(${{{}}})", ids_arg),
                format!("ResourceAddress(\"${{{}}}\")", resource_address_arg),
            ],
        };

        self.needed_resources.push(inst);
    }

    fn create_admin_badge_proof(&mut self, account_arg: String, resource_address_arg: String) {
        let inst = Instruction::CallMethod {
            component_address_arg: account_arg,
            method_name: "create_proof".to_string(),
            args: vec![format!(
                "ResourceAddress(\"${{{}}}\")",
                resource_address_arg
            )],
        };
        self.needed_resources.push(inst);
    }

    fn create_fungible_proof(
        &mut self,
        account_arg: String,
        resource_address_arg: String,
        amount_arg: String,
    ) {
        let inst = Instruction::CallMethod {
            component_address_arg: account_arg,
            method_name: "create_proof_by_amount".to_string(),
            args: vec![
                format!("Decimal(\"${{{}}}\")", amount_arg),
                format!("ResourceAddress(\"${{{}}}\")", resource_address_arg),
            ],
        };
        self.needed_resources.push(inst);
    }

    pub fn create_non_fungible_proof(
        &mut self,
        account_arg: String,
        resource_address_arg: String,
        id_arg: String,
    ) {
        let resource_arg = format!("ResourceAddress(\"${{{}}}\")", resource_address_arg);
        let inst = Instruction::CallMethod {
            component_address_arg: account_arg,
            method_name: "create_proof_by_ids".to_string(),
            args: vec![
                format!("Array<NonFungibleId>(${{{}}})", id_arg),
                resource_arg,
            ],
        };
        self.needed_resources.push(inst);
    }

    fn create_usable_fungible_proof(
        &mut self,
        account_arg: String,
        resource_address_arg: String,
        amount_arg: String,
        proof_id: u32,
    ) {
        self.create_fungible_proof(
            account_arg,
            resource_address_arg.clone(),
            amount_arg.clone(),
        );

        let inst = Instruction::CreateProofFromAuthZoneByAmount {
            amount_arg,
            resource_address_arg,
            proof_id,
        };

        self.needed_resources.push(inst);
    }

    fn create_usable_non_fungible_proof(
        &mut self,
        account_arg: String,
        resource_address_arg: String,
        ids_arg: String,
        proof_id: u32,
    ) {
        self.create_non_fungible_proof(account_arg, resource_address_arg.clone(), ids_arg.clone());

        let inst = Instruction::CreateProofFromAuthZoneByIds {
            ids_arg,
            resource_address_arg,
            proof_id,
        };
        self.needed_resources.push(inst);
    }

    fn deposit_batch(&mut self, account_arg: String) {
        let inst = Instruction::CallMethod {
            component_address_arg: account_arg,
            method_name: "deposit_batch".to_string(),
            args: vec![String::from("Expression(\"ENTIRE_WORKTOP\")")],
        };

        self.instructions.push(inst);
    }

    fn drop_proofs(&mut self) {
        if self.has_proofs {
            let inst = Instruction::DropAllProofs;
            self.instructions.push(inst);
        }
    }

    fn deal_with_args(&mut self, args: &Vec<Arg>) -> Vec<String> {
        let mut args_vec = vec![];

        for arg in args {
            match arg {
                Arg::FungibleBucketArg(_, _) => {
                    let amount_arg = format!("arg_{}_amount", self.arg_count);
                    let resource_arg = format!("arg_{}_resource", self.arg_count);
                    self.withdraw_by_amount(
                        Self::caller_arg(),
                        amount_arg.clone(),
                        resource_arg.clone(),
                    );
                    self.take_from_worktop_by_amount(amount_arg, resource_arg, self.id);
                    let ret = format!("Bucket(\"{}\")", self.id);
                    self.id += 1;
                    args_vec.push(ret);
                }
                Arg::NonFungibleBucketArg(_, _) => {
                    let resource_arg = format!("arg_{}_resource", self.arg_count);
                    let ids_arg = format!("arg_{}_ids", self.arg_count);
                    self.withdraw_by_ids(Self::caller_arg(), resource_arg.clone(), ids_arg.clone());
                    self.take_from_worktop_by_ids(resource_arg, ids_arg, self.id);
                    let ret = format!("Bucket(\"{}\")", self.id);
                    self.id += 1;
                    args_vec.push(ret);
                }
                Arg::FungibleProofArg(_, _) => {
                    let amount_arg = format!("arg_{}_amount", self.arg_count);
                    let resource_arg = format!("arg_{}_resource", self.arg_count);
                    self.create_usable_fungible_proof(
                        Self::caller_arg(),
                        resource_arg,
                        amount_arg,
                        self.id,
                    );
                    let ret = format!("Proof(\"{}\")", self.id);
                    self.id += 1;
                    self.has_proofs = true;
                    args_vec.push(ret);
                }
                Arg::NonFungibleProofArg(_, _) => {
                    let ids_arg = format!("arg_{}_ids", self.arg_count);
                    let resource_arg = format!("arg_{}_resource", self.arg_count);
                    self.create_usable_non_fungible_proof(
                        Self::caller_arg(),
                        resource_arg,
                        ids_arg,
                        self.id,
                    );
                    let ret = format!("Proof(\"{}\")", self.id);
                    self.id += 1;
                    self.has_proofs = true;
                    args_vec.push(ret);
                }
                Arg::NonFungibleAddressArg(_, _) => {
                    let resource_arg = format!("arg_{}_resource", self.arg_count);
                    let id_arg = format!("arg_{}_id", self.arg_count);
                    let ret = format!("{}(\"{}\", {})", arg.get_type(), resource_arg, id_arg);
                    args_vec.push(ret);
                }
                _ => {
                    args_vec.push(arg.to_generic(self.arg_count));
                }
            }
            self.arg_count += 1;
        }

        args_vec
    }

    pub fn build(&self) -> String {
        let mut output = String::new();
        for instr in &self.needed_resources {
            output = format!("{}\n\n{}", output, instr);
        }
        for instr in &self.instructions {
            output = format!("{}\n\n{}", output, instr);
        }

        output
    }

    pub fn caller_arg() -> String {
        String::from("caller_address")
    }

    pub fn component_arg() -> String {
        String::from("component_address")
    }

    pub fn admin_badge_arg() -> String {
        String::from("badge_address")
    }

    pub fn package_arg() -> String {
        String::from("package_address")
    }
}
