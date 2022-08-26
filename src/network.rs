use crate::bank::Banks;
use crate::bank::ID;
use crate::bank::NUM_NODES;
use crate::node::Node;
use crate::tower::Slot;
use crate::tower::Vote;
//use rayon::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

pub struct Network {
    nodes: Vec<Node>,
    banks: Banks,
    slot: Slot,
    num_partitions: usize,
    partitioned_blocks: VecDeque<(ID, Slot)>,
}
impl Default for Network {
    fn default() -> Self {
        let mut nodes = vec![];
        for i in 0..NUM_NODES {
            nodes.push(Node::zero(i));
        }
        Network {
            banks: Banks::default(),
            nodes,
            slot: 0,
            num_partitions: 0,
            partitioned_blocks: VecDeque::new(),
        }
    }
}
impl Network {
    fn hash(val: u64) -> u64 {
        let mut h = DefaultHasher::new();
        val.hash(&mut h);
        h.finish()
    }
    fn check_same_partition(num_partitions: usize, a: ID, b: ID) -> bool {
        num_partitions == 0 || a % num_partitions == b % num_partitions
    }
    pub fn create_partitions(&mut self, num: usize) {
        self.num_partitions = num;
    }
    pub fn repair_partitions(&mut self, new_partitions: usize) {
        for (block_producer_ix, block) in &self.partitioned_blocks {
            self.nodes.iter_mut().enumerate().for_each(|(i, n)| {
                //already delivered
                if new_partitions == 0 || Self::check_same_partition(new_partitions, *block_producer_ix, i){
                    n.set_active_block(*block);
                }
            });
        }
        self.num_partitions = new_partitions;
    }
    pub fn root(&self) -> Vote {
        self.banks.lowest_root
    }
    pub fn step(&mut self) {
        self.slot = self.slot + 1;
        println!("slot {} voting", self.slot);
        self.nodes.iter_mut().for_each(|n| n.vote(&self.banks));
        let block_producer_ix = Self::hash(self.slot) as usize % self.nodes.len();
        let block_producer = &self.nodes[block_producer_ix];
        let votes: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if !Self::check_same_partition(self.num_partitions, block_producer_ix, i) {
                    return None;
                }
                let votes = n.votes();
                Some((i, votes))
            })
            .collect();
        let block = block_producer.make_block(self.slot, votes);
        self.banks.apply(&block);
        self.nodes.iter_mut().enumerate().for_each(|(i, n)| {
            if Self::check_same_partition(self.num_partitions, block_producer_ix, i) {
                n.set_active_block(self.slot);
            }
        });
        if self.num_partitions > 0 {
            self.partitioned_blocks
                .push_back((block_producer_ix, block.slot));
        }
        let root_slot = self.root().slot;
        self.partitioned_blocks.retain(|(_, b)| *b >= root_slot);
    }
}
