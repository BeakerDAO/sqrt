use radix_engine::ledger::OutputId;
use radix_engine::types::Hash;
use radix_engine_stores::hash_tree::tree_store::{TypedInMemoryTreeStore, Version};
use radix_engine_stores::hash_tree::{put_at_next_version, SubstateHashChange};

pub struct StateHashSupport {
    tree_store: TypedInMemoryTreeStore,
    current_version: Version,
    current_hash: Hash,
}

impl StateHashSupport {
    pub fn new() -> Self {
        StateHashSupport {
            tree_store: TypedInMemoryTreeStore::new(),
            current_version: 0,
            current_hash: Hash([0; Hash::LENGTH]),
        }
    }

    pub fn update_with(&mut self, transaction_outputs: Vec<OutputId>) {
        let hash_changes = transaction_outputs
            .iter()
            .map(|output_id| {
                SubstateHashChange::new(
                    output_id.substate_id.clone(),
                    Some(output_id.substate_hash),
                )
            })
            .collect::<Vec<_>>();
        self.current_hash = put_at_next_version(
            &mut self.tree_store,
            Some(self.current_version).filter(|version| *version > 0),
            hash_changes,
        );
        self.current_version += 1;
    }
}
