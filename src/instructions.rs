use std::fmt::{Display, Formatter};
use sbor::{Decode, Encode, TypeId};
use scrypto::component::ComponentAddress;
use scrypto::prelude::Decimal;
use scrypto::resource::ResourceAddress;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Instruction {
    TakeFromWorktopByAmount{
        amount: Decimal,
        resource_address: ResourceAddress,
        bucket_id: u32
    },

    CallMethod{
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<String>
    },

    CreateProofFromAuthZone{
        resource_address: ResourceAddress,
        proof_id: u32
    },

    DropAllProofs
}

impl Display for Instruction
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Instruction::TakeFromWorktopByAmount
            { amount, resource_address, bucket_id } =>
                {
                    write!(f, "TAKE_FROM_WORKTOP_BY_AMOUNT\n\
                               \tDecimal(\"{}\")\n\
                               \tResourceAddress(\"{}\")\n\
                               \tBucket(\"{}\");", amount, resource_address, bucket_id)
                }

            Instruction::CallMethod
            { component_address, method_name, args } =>
                {
                    let mut arg_str = String::new();
                    for arg in args
                    {
                        arg_str = format!("{}\n\
                                           \t{}", arg_str, arg);
                    }
                    write!(f, "CALL_METHOD\n\
                               \tComponentAddress(\"{}\")\n\
                               \t\"{}\"\
                               {};", component_address, method_name, arg_str)
                }
            Instruction::CreateProofFromAuthZone { resource_address, proof_id } =>
                {
                    write!(f, "CREATE_PROOF_FROM_AUTH_ZONE\n\
                              \tResourceAddress(\"{}\")\n\
                              \tProof(\"{}\");", resource_address, proof_id)
                }

            Instruction::DropAllProofs =>
                {
                    write!(f, "DROP_ALL_PROOFS;")
                }
        }
    }
}