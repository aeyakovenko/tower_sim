use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;
use std::collections::HashSet;

pub const NUM_NODES: usize = 997;
pub type ID = usize;

pub struct Bank {
    pub nodes: Vec<Tower>,
    pub slot: Slot,
    pub parent: Slot,
    pub children: Vec<Slot>,
}

pub struct Block {
    pub slot: Slot,
    pub parent: Slot,
    pub votes: Vec<(ID, Vec<Vote>)>,
}

pub struct Banks {
    pub fork_map: HashMap<Slot, Bank>,
    pub fork_weights: HashMap<Slot, usize>,
    pub lowest_root: Vote,
}

impl Default for Banks {
    fn default() -> Self {
        let bank_zero = Bank::zero();
        let mut fork_map = HashMap::new();
        fork_map.insert(0, bank_zero);
        Self {
            fork_map,
            fork_weights: HashMap::new(),
            lowest_root: Vote::zero(),
        }
    }
}

impl Banks {
    pub fn apply(&mut self, block: &Block) {
        assert!(self.fork_map.get(&block.slot).is_none());
        let parent = self.fork_map.get_mut(&block.parent).unwrap();
        let mut bank = parent.child(block.slot);
        let mut fork: HashSet<_> = self.compute_fork(block.parent).into_iter().collect();
        fork.insert(bank.slot);
        bank.apply(block, &fork);
        let lowest_root = bank.lowest_root();
        assert!(self.fork_map.get(&bank.slot).is_none());
        let mut max_root = 0;
        for n in bank.nodes.iter() {
            if max_root > n.root.slot && max_root - n.root.slot > 100 {
                println!("here");
            }
            if n.root.slot > max_root {
                max_root = n.root.slot;
            }
        }
        println!("ROOT DISTANCE {}", max_root - lowest_root.slot);
        self.fork_map.insert(bank.slot, bank);
        if lowest_root.slot > self.lowest_root.slot {
            println!(
                "LOWEST ROOT UPDATE {:?} {:?} MAX: {}",
                self.lowest_root, lowest_root, max_root
            );
            self.lowest_root = lowest_root;
            self.gc();
        }
        self.build_fork_weights();
    }

    pub fn compute_fork(&self, slot: Slot) -> Vec<Slot> {
        let mut fork = vec![slot];
        loop {
            let last = fork.last().unwrap();
            if let Some(b) = self.fork_map.get(last) {
                if *last == b.parent {
                    break;
                }
                fork.push(b.parent)
            } else {
                break;
            }
        }
        fork
    }

    //only keep forks that are connected to root
    fn gc(&mut self) {
        let mut valid = vec![];

        println!("START GC {:?}", self.lowest_root);
        let mut children = vec![self.lowest_root.slot];
        while !children.is_empty() {
            let slot = children.pop().unwrap();
            valid.push(slot);
            let bank = self.fork_map.get(&slot).unwrap();
            children.extend_from_slice(&bank.children);
        }
        let mut new_banks = HashMap::new();
        for v in valid {
            new_banks.insert(v, self.fork_map.remove(&v).unwrap());
        }
        self.fork_map = new_banks;
    }
    /// A validator V's vote on an ancestor X counts towards a descendant
    /// Y even if the validator is not locked out on X at Y anymore,
    /// as long as X is the latest vote observed from this validator V
    pub fn build_fork_weights(&mut self) {
        //each validators latest votes
        let mut latest_votes: HashMap<ID, Slot> = HashMap::new();
        for v in self.fork_map.values() {
            v.latest_votes(&mut latest_votes);
        }
        //total stake voting per slot
        let mut slot_votes: HashMap<Slot, usize> = HashMap::new();
        for (_, v) in &latest_votes {
            let e = slot_votes.entry(*v).or_insert(0);
            *e = *e + 1;
        }
        //stake weight is inherited from the parent
        let mut weights: HashMap<Slot, usize> = HashMap::new();
        let mut children = vec![self.lowest_root.slot];
        while !children.is_empty() {
            let child = children.pop().unwrap();
            let bank = self.fork_map.get(&child).unwrap();
            children.extend_from_slice(&bank.children);
            let parent_weight = *weights.get(&bank.parent).unwrap_or(&0);
            let e = weights.entry(child).or_insert(parent_weight);
            *e = *e + *slot_votes.get(&child).unwrap_or(&0);
        }
        self.fork_weights = weights;
    }
}

impl Bank {
    pub fn zero() -> Self {
        let mut nodes = vec![];
        for _ in 0..NUM_NODES {
            nodes.push(Tower::default());
        }
        Bank {
            nodes,
            slot: 0,
            parent: 0,
            children: vec![],
        }
    }
    pub fn child(&mut self, slot: Slot) -> Self {
        let b = Bank {
            nodes: self.nodes.clone(),
            slot,
            parent: self.slot,
            children: vec![],
        };
        self.children.push(slot);
        b
    }
    pub fn apply(&mut self, block: &Block, fork: &HashSet<Slot>) {
        assert_eq!(self.slot, block.slot);
        assert_eq!(self.parent, block.parent);
        for (id, votes) in &block.votes {
            for v in votes {
                assert!(
                    fork.contains(&v.slot),
                    "proposed vote is not in the bank's fork {:?} {}",
                    fork,
                    v.slot
                );
                let _e = self.nodes[*id].apply(v);
            }
        }
    }

    pub fn print_threshold_slot(&self, mult: u64, vote: &Vote) {
        self.nodes.iter().enumerate().for_each(|(i, n)| {
            //alredy rooted
            if n.root.slot >= vote.slot {
                return;
            }
            for v in &n.votes {
                //check if the node has a higher vote with at least 1/2 the lockout
                if v.slot >= vote.slot
                    && (v.slot + (mult * v.lockout)) >= (vote.slot + vote.lockout)
                {
                    return;
                }
                if v.slot == vote.slot {
                    println!("{} {:?}", i, v);
                }
            }
        });
    }

    pub fn calc_threshold_slot(&self, mult: u64, vote: &Vote) -> usize {
        let count: usize = self
            .nodes
            .iter()
            .map(|n| {
                //alredy rooted
                if n.root.slot >= vote.slot {
                    return 1;
                }
                for v in &n.votes {
                    //check if the node has a higher vote with at least 1/2 the lockout
                    if v.slot >= vote.slot
                        && (v.slot + (mult * v.lockout)) >= (vote.slot + vote.lockout)
                    {
                        return 1;
                    }
                }
                0
            })
            .sum();
        count
    }
    pub fn threshold_slot(&self, vote: &Vote) -> bool {
        self.calc_threshold_slot(2, vote) > (2 * NUM_NODES) / 3
    }
    pub fn supermajority_root(&self) -> Vote {
        let mut roots: Vec<_> = self.nodes.iter().map(|n| n.root).collect();
        roots.sort_by_key(|x| x.slot);
        //2/3 of the nodes are at least at this root
        roots[NUM_NODES / 3]
    }

    fn lowest_root(&self) -> Vote {
        let mut roots: Vec<_> = self.nodes.iter().map(|n| n.root).collect();
        roots.sort_by_key(|x| x.slot);
        roots[0]
    }

    //get the latest votes from each node
    pub fn latest_votes(&self, latest_votes: &mut HashMap<ID, Slot>) {
        for (i, n) in self.nodes.iter().enumerate() {
            let latest = n.latest_vote().unwrap_or(&n.root);
            let e = latest_votes.entry(i).or_insert(latest.slot);
            if *e < latest.slot {
                *e = latest.slot;
            }
        }
    }
}
