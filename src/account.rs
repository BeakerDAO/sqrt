use radix_engine::types::{ComponentAddress, EcdsaSecp256k1PublicKey};
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

pub struct Account {
    private_key: EcdsaSecp256k1PrivateKey,
    public_key: EcdsaSecp256k1PublicKey,
    component_address: ComponentAddress,
    owner_badge: Option<NonFungibleGlobalId>
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
            owner_badge: None
        }
    }

    pub fn address(&self) -> ComponentAddress
    {
        self.component_address.clone()
    }

    pub fn owner_badge(&self) -> NonFungibleGlobalId{
        self.owner_badge.clone().expect("The account does not have an owner badge")
    }

    pub fn set_owner_badge(&mut self, owner_badge: NonFungibleGlobalId) {
        match self.owner_badge {
            None => self.owner_badge = Some(owner_badge),
            Some(_) => panic!("The account already has an owner badge")
        }
    }
}
