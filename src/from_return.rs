use radix_engine::types::{ComponentAddress, Decimal, Hash, NonFungibleGlobalId, NonFungibleLocalId, PackageAddress, PreciseDecimal, ResourceAddress};
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::scrypto_decode;
use radix_engine_interface::data::scrypto::ScryptoDecode;

pub trait FromReturn: ScryptoDecode {

    fn from(instructions: Vec<InstructionOutput>) -> Self;

}

macro_rules! from_return_impl {
    ($type:ident) => {
        impl FromReturn for $type {
            fn from(mut instructions: Vec<InstructionOutput>) -> Self{
                if instructions.len() != 1 {
                    panic!("Could not parse method return into given type")
                }

                let bytes = match instructions.pop().unwrap() {
                    InstructionOutput::None => { panic!("The method does not return anything") }
                    InstructionOutput::CallReturn(bytes) => bytes
                };
                scrypto_decode::<$type>(&bytes).expect("Could not parse method return into given type")
            }
        }
    }
}

from_return_impl!(u8);
from_return_impl!(u16);
from_return_impl!(u32);
from_return_impl!(u64);
from_return_impl!(u128);
from_return_impl!(i8);
from_return_impl!(i16);
from_return_impl!(i32);
from_return_impl!(i64);
from_return_impl!(i128);
from_return_impl!(String);
from_return_impl!(ComponentAddress);
from_return_impl!(PackageAddress);
from_return_impl!(ResourceAddress);
from_return_impl!(NonFungibleGlobalId);
from_return_impl!(NonFungibleLocalId);
from_return_impl!(Hash);
from_return_impl!(Decimal);
from_return_impl!(PreciseDecimal);


macro_rules! from_return_tuple_impl {
    ( $($type:ident)+ ) => {
        impl<$($type: FromReturn),+> FromReturn for ($($type, )+) {
            fn from(instructions: Vec<InstructionOutput>) -> Self{
                let mut i = 0;
                (
                    $(
                        {
                            let bytes = match instructions.get(i).clone().unwrap() {
                            InstructionOutput::None => { panic!("The method does not return anything") }
                            InstructionOutput::CallReturn(bytes) => bytes
                            };

                            let elem = scrypto_decode::<$type>(&bytes).expect("Could not parse method return into given type");
                            i = i+1;
                            elem
                        }
                    ),*
                )
            }
        }
    }
}

from_return_tuple_impl!(A B);
from_return_tuple_impl!(A B C);
from_return_tuple_impl!(A B C D);
from_return_tuple_impl!(A B C D E);
from_return_tuple_impl!(A B C D E F);
from_return_tuple_impl!(A B C D E F G);
from_return_tuple_impl!(A B C D E F G H);
from_return_tuple_impl!(A B C D E F G H I);
from_return_tuple_impl!(A B C D E F G H I J);
from_return_tuple_impl!(A B C D E F G H I J K);
from_return_tuple_impl!(A B C D E F G H I J K L);