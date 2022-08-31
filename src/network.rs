use crate::bank::ID;
use crate::bank::NUM_NODES;
use crate::forks::Forks;
use crate::node::Node;
use crate::subcommittee::hash;
use crate::tower::Slot;
use crate::tower::Vote;
use std::collections::HashSet;
//use rayon::prelude::*;
use std::collections::VecDeque;

pub struct Network {
    nodes: Vec<Node>,
    forks: Forks,
    slot: Slot,
    num_partitions: usize,
    partitioned_blocks: VecDeque<(ID, Slot)>,
    oc_slots: HashSet<Slot>,
}
impl Default for Network {
    fn default() -> Self {
        let mut nodes = vec![];
        for i in 0..NUM_NODES {
            nodes.push(Node::zero(i));
        }
        Network {
            forks: Forks::default(),
            nodes,
            slot: 0,
            num_partitions: 0,
            partitioned_blocks: VecDeque::new(),
            oc_slots: HashSet::new(),
        }
    }
}
impl Network {
    fn check_same_partition(num_partitions: usize, a: ID, b: ID) -> bool {
        num_partitions == 0 || (a % num_partitions == b % num_partitions)
    }
    pub fn create_partitions(&mut self, num: usize) {
        self.num_partitions = num;
    }
    pub fn repair_partitions(&mut self, new_partitions: usize) {
        for (block_producer_ix, block) in &self.partitioned_blocks {
            self.nodes.iter_mut().enumerate().for_each(|(i, n)| {
                if Self::check_same_partition(new_partitions, *block_producer_ix, i) {
                    n.set_active_block(*block);
                }
            });
        }
        self.num_partitions = new_partitions;
    }
    pub fn lowest_root(&self) -> Vote {
        self.forks.lowest_root
    }
    pub fn step(&mut self) {
        self.slot = self.slot + 1;
        println!("slot {} voting", self.slot);
        self.nodes.iter_mut().for_each(|n| n.vote(&self.forks));
        let block_producer_ix = hash(self.slot) as usize % self.nodes.len();
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
        self.forks.apply(&block);
        let oc_slots = self.forks.fork_map.get(&block.slot).unwrap().oc_slots();
        self.oc_slots.extend(&oc_slots);
        self.nodes.iter_mut().enumerate().for_each(|(i, n)| {
            if Self::check_same_partition(self.num_partitions, block_producer_ix, i) {
                n.set_active_block(self.slot);
            }
        });
        if self.num_partitions > 0 {
            self.partitioned_blocks
                .push_back((block_producer_ix, block.slot));
        }
        let lowest_root = self.lowest_root().slot;
        self.partitioned_blocks.retain(|(_, b)| *b >= lowest_root);
        println!("OC SLOTS {:?}", self.oc_slots);
        self.oc_slots.retain(|s| !self.forks.roots.contains(s));
        for s in &self.oc_slots {
            assert!(*s >= lowest_root);
        }
    }
}
