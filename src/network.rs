use crate::bank::Block;
use crate::bank::ID;
use crate::bank::NUM_NODES;
use crate::node::Node;
use crate::tower::Slot;
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
    fn check_same_partition(&self, a: ID, b: ID) -> bool {
        self.num_partitions == 0 || a % self.num_partitions == b % self.num_partitions
    }
    pub fn create_partitions(&mut self, num: usize) {
        self.num_partitions = num;
    }
    pub fn repair_partitions(&mut self) {
        for (block_producer_ix, block) in &self.partitioned_blocks {
            for i in 0..self.nodes.len() {
                //already delivered
                if self.check_same_partition(*block_producer_ix, i) {
                    continue;
                }
                self.nodes[i].apply(block);
            }
        }
        self.num_partitions = 0;
        self.partitioned_blocks = VecDeque::new();
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
            if !self.check_same_partition(block_producer_ix, i) {
                continue;
            }
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
        for i in 0..self.nodes.len() {
            if !self.check_same_partition(block_producer_ix, i) {
                continue;
            }
            self.nodes[i].apply(&block);
        }
        if self.num_partitions > 0 {
            self.partitioned_blocks
                .push_back((block_producer_ix, block));
        }
    }
}
