use radix_engine::types::{ComponentAddress, EcdsaSecp256k1PublicKey};
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

pub struct Account {
    private_key: EcdsaSecp256k1PrivateKey,
    public_key: EcdsaSecp256k1PublicKey,
    component_address: ComponentAddress,
}

impl Account {
    pub fn new(
        private_key: EcdsaSecp256k1PrivateKey,
        public_key: EcdsaSecp256k1PublicKey,
        component_address: ComponentAddress,
    ) -> Self {
        Self {
            private_key,
            public_key,
            component_address,
        }
    }
}
