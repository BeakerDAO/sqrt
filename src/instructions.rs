use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    CallFunction {
        package_address_arg: String,
        blueprint_name_arg: String,
        function_name_arg: String,
        args: Vec<String>,
    },

    CallMethod {
        component_address_arg: String,
        method_name: String,
        args: Vec<String>,
    },

    CreateProofFromAuthZoneByAmount {
        amount_arg: String,
        resource_address_arg: String,
        proof_id: u32,
    },

    CreateProofFromAuthZoneByIds {
        ids_arg: String,
        resource_address_arg: String,
        proof_id: u32,
    },

    DropAllProofs,

    TakeFromWorktopByAmount {
        amount_arg: String,
        resource_address_arg: String,
        bucket_id: u32,
    },

    TakeFromWorktopByIds {
        ids_arg: String,
        resource_address_arg: String,
        bucket_id: u32,
    },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::CallFunction {
                package_address_arg,
                blueprint_name_arg,
                function_name_arg,
                args,
            } => {
                let mut arg_str = String::new();
                for arg in args {
                    arg_str = format!(
                        "{}\n\
                         \t{}",
                        arg_str, arg
                    );
                }
                write!(
                    f,
                    "CALL_FUNCTION\n\
                               \tPackageAddress(\"${{{}}}\")\n\
                               \t\"{}\"\n\
                               \t\"{}\"\
                               {};",
                    package_address_arg, blueprint_name_arg, function_name_arg, arg_str
                )
            }

            Instruction::CallMethod {
                component_address_arg,
                method_name,
                args,
            } => {
                let mut arg_str = String::new();
                for arg in args {
                    arg_str = format!(
                        "{}\n\
                         \t{}",
                        arg_str, arg
                    );
                }
                write!(
                    f,
                    "CALL_METHOD\n\
                               \tComponentAddress(\"${{{}}}\")\n\
                               \t\"{}\"\
                               {};",
                    component_address_arg, method_name, arg_str
                )
            }
            Instruction::CreateProofFromAuthZoneByAmount {
                amount_arg,
                resource_address_arg,
                proof_id,
            } => {
                write!(
                    f,
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_AMOUNT\n\
                              \tDecimal(\"${{{}}}\")
                              \tResourceAddress(\"${{{}}}\")\n\
                              \tProof(\"{}\");",
                    amount_arg, resource_address_arg, proof_id
                )
            }

            Instruction::CreateProofFromAuthZoneByIds {
                ids_arg,
                resource_address_arg,
                proof_id,
            } => {
                write!(
                    f,
                    "CREATE_PROOF_FROM_AUTH_ZONE_BY_IDS\n\
                               \tArray<NonFungibleId>(${{{}}})\n\
                               \tResourceAddress(\"${{{}}}\")\n\
                               \tProof(\"{}\");",
                    ids_arg, resource_address_arg, proof_id
                )
            }

            Instruction::DropAllProofs => {
                write!(f, "DROP_ALL_PROOFS;")
            }
            Instruction::TakeFromWorktopByAmount {
                amount_arg,
                resource_address_arg,
                bucket_id,
            } => {
                write!(
                    f,
                    "TAKE_FROM_WORKTOP_BY_AMOUNT\n\
                               \tDecimal(\"${{{}}}\")\n\
                               \tResourceAddress(\"${{{}}}\")\n\
                               \tBucket(\"{}\");",
                    amount_arg, resource_address_arg, bucket_id
                )
            }

            Instruction::TakeFromWorktopByIds {
                resource_address_arg,
                ids_arg,
                bucket_id,
            } => {
                write!(
                    f,
                    "TAKE_FROM_WORKTOP_BY_IDS\n\
                               \tArray<NonFungibleId>(${{{}}})\n\
                               \tResourceAddress(\"${{{}}}\")\n\
                               \tBucket(\"{}\");",
                    ids_arg, resource_address_arg, bucket_id
                )
            }
        }
    }
}
