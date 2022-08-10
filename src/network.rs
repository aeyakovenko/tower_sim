use crate::bank::Block;
use crate::bank::NUM_NODES;
use crate::node::Node;
use crate::tower::Slot;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

struct Network {
    nodes: [Node; NUM_NODES],
    slot: Slot,
}

impl Network {
    fn hash(val: u64) -> u64 {
        let mut h = DefaultHasher::new();
        val.hash(&mut h);
        h.finish()
    }
    pub fn step(&mut self) {
        self.slot = self.slot + 1;
        for n in &mut self.nodes {
            n.vote();
        }
        let block_producer_ix = Self::hash(self.slot) as usize % self.nodes.len();
        let block_producer = &self.nodes[block_producer_ix];
        let heaviest_fork = &block_producer.heaviest_fork;
        let mut votes = vec![];
        for (i, n) in self.nodes.iter().enumerate() {
            let vote = n.last_vote();
            if heaviest_fork.iter().find(|x| **x == vote.slot).is_some() {
                votes.push((i, vote.clone()))
            }
        }
        let block = Block {
            slot: self.slot,
            parent: *heaviest_fork.get(0).unwrap_or(&0),
            votes,
        };
        for n in &mut self.nodes {
            n.apply(&block);
        }
    }
}
