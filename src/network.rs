use crate::bank::Block;
use crate::bank::ID;
use crate::bank::NUM_NODES;
use crate::node::Node;
use crate::tower::Slot;
use crate::tower::Vote;
use rayon::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

pub struct Network {
    nodes: Vec<Node>,
    slot: Slot,
    num_partitions: usize,
    partitioned_blocks: VecDeque<(ID, Block)>,
}
impl Default for Network {
    fn default() -> Self {
        let mut nodes = vec![];
        for i in 0..NUM_NODES {
            nodes.push(Node::zero(i));
        }
        Network {
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
    pub fn repair_partitions(&mut self) {
        for (block_producer_ix, block) in &self.partitioned_blocks {
            self.nodes.par_iter_mut().enumerate().for_each(|(i, n)| {
                //already delivered
                if !Self::check_same_partition(self.num_partitions, *block_producer_ix, i) {
                    n.apply(block);
                }
            });
        }
        self.num_partitions = 0;
        self.partitioned_blocks = VecDeque::new();
    }
    pub fn root(&self) -> Option<Vote> {
        self.nodes.iter().map(|n| n.supermajority_root).min()
    }
    pub fn step(&mut self) {
        self.slot = self.slot + 1;
        println!("slot {} voting", self.slot);
        self.nodes.par_iter_mut().for_each(|n| n.vote());
        let block_producer_ix = Self::hash(self.slot) as usize % self.nodes.len();
        println!("bp {}", block_producer_ix);
        let block_producer = &self.nodes[block_producer_ix];
        let votes: Vec<_> = self
            .nodes
            .par_iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if !Self::check_same_partition(self.num_partitions, block_producer_ix, i) {
                    return None;
                }
                let vote = n.last_vote();
                Some((i, vote.clone()))
            })
            .collect();
        let block = block_producer.make_block(self.slot, votes);
        self.nodes.par_iter_mut().enumerate().for_each(|(i, n)| {
            if Self::check_same_partition(self.num_partitions, block_producer_ix, i) {
                n.apply(&block);
            }
        });
        if self.num_partitions > 0 {
            self.partitioned_blocks
                .push_back((block_producer_ix, block));
        }
    }
}
