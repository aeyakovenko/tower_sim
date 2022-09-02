use crate::bank::ID;
use crate::bank::NUM_NODES;
use crate::forks::Forks;
use crate::node::Node;
use crate::subcommittee::hash;
use crate::tower::Slot;
use crate::tower::Vote;
use rayon::prelude::*;
use std::collections::HashSet;
use std::collections::VecDeque;

pub struct Network {
    nodes: Vec<Node>,
    forks: Forks,
    slot: Slot,
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
            partitioned_blocks: VecDeque::new(),
            oc_slots: HashSet::new(),
        }
    }
}
impl Network {
    pub fn partition_step(
        &mut self,
        partitions: &[(usize, usize)],
        active: &[bool],
        block_producer_ix: usize,
    ) {
        self.repair_partitions(partitions, active);
        self.vote(partitions, active);
        let block_producer = &self.nodes[block_producer_ix];

        let votes: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if !Self::check_same_partition(partitions, active, block_producer_ix, i) {
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
            if Self::check_same_partition(partitions, active, block_producer_ix, i) {
                n.set_active_block(self.slot);
            }
        });
        let num_dead_partitions: usize = active.iter().map(|x| !(*x) as usize).sum();
        if num_dead_partitions > 0 {
            self.partitioned_blocks
                .push_back((block_producer_ix, block.slot));
        }
        let lowest_root = self.lowest_root().slot;
        self.partitioned_blocks.retain(|(_, b)| *b >= lowest_root);
        println!("OC SLOTS {:?}", self.oc_slots);
        self.oc_slots.retain(|s| !self.forks.roots.contains(s));
        for s in &self.oc_slots {
            assert!(*s >= lowest_root, "OC failed {}", *s);
        }
    }

    pub fn step(&mut self, num_partitions: usize) {
        let block_producer_ix = hash(self.slot) as usize % self.nodes.len();
        let mut partitions = vec![];
        for i in 1..num_partitions {
            let num = NUM_NODES / num_partitions;
            assert!(num > 0, "invalid number of partitions");
            let min = (i - 1) * num;
            let max = std::cmp::max(NUM_NODES, i * num);
            partitions.push((min, max));
        }
        let mut active = vec![];
        for (s, e) in &partitions {
            if block_producer_ix >= *s && block_producer_ix < *e {
                active.push(true);
            } else {
                active.push(false);
            }
        }
        self.partition_step(&partitions, &active, block_producer_ix);
    }

    fn check_same_partition(partitions: &[(usize, usize)], active: &[bool], a: ID, b: ID) -> bool {
        if partitions.is_empty() {
            return true;
        }
        let mut a_active = false;
        let mut b_active = false;
        for (r, (s, e)) in active.iter().zip(partitions) {
            if *r && a >= *s && a < *e {
                a_active = true;
            }
            if *r && b >= *s && b < *e {
                b_active = true;
            }
            if a_active && b_active {
                return true;
            }
        }
        a_active && b_active
    }

    fn repair_partitions(&mut self, partitions: &[(usize, usize)], active: &[bool]) {
        for (bp, slot) in &self.partitioned_blocks {
            for (id, n) in self.nodes.iter_mut().enumerate() {
                if Self::check_same_partition(partitions, active, *bp, id) {
                    n.set_active_block(*slot);
                }
            }
        }
    }

    fn vote(&mut self, partitions: &[(usize, usize)], active: &[bool]) {
        for (r, (s, e)) in active.iter().zip(partitions) {
            if *r {
                self.nodes[*s..*e]
                    .par_iter_mut()
                    .for_each(|n| n.vote(&self.forks));
            }
        }
    }

    pub fn lowest_root(&self) -> Vote {
        self.forks.lowest_root
    }
}
