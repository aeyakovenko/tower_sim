use crate::bank::{Bank, Block, ID};
use crate::subcommittee::Phase;
use crate::tower::{Slot, Vote};
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Forks {
    pub fork_map: HashMap<Slot, Bank>,
    pub primary_fork_weights: HashMap<Slot, usize>,
    pub lowest_root: Vote,
    pub roots: HashSet<Slot>,
}

impl Default for Forks {
    fn default() -> Self {
        let bank_zero = Bank::zero();
        let mut fork_map = HashMap::new();
        fork_map.insert(0, bank_zero);
        let mut roots = HashSet::new();
        roots.insert(0);
        Self {
            roots,
            fork_map,
            primary_fork_weights: HashMap::new(),
            lowest_root: Vote::zero(),
        }
    }
}

impl Forks {
    pub fn apply(&mut self, block: &Block) {
        assert!(self.fork_map.get(&block.slot).is_none());
        let parent = self.fork_map.get_mut(&block.parent).unwrap();
        let parent_phase = parent.subcom.phase();
        let mut bank = parent.child(block.slot);
        let mut fork: HashSet<_> = self.compute_fork(block.parent).into_iter().collect();
        fork.insert(bank.slot);
        bank.apply(block, &fork);

        if Phase::FlipPrimary == bank.subcom.phase() && parent_phase != Phase::FlipPrimary {
            let primary = bank.primary_super_root().slot;
            let secondary = bank.secondary_super_root().slot;
            let s = self.compute_fork(secondary);
            let p = self.compute_fork(primary);
            if secondary >= self.lowest_root.slot && primary >= self.lowest_root.slot {
                if secondary > primary {
                    assert!(
                        s.contains(&primary),
                        "{} diverged {:?} {}",
                        self.lowest_root.slot,
                        s,
                        primary
                    );
                }
                if secondary < primary {
                    assert!(
                        p.contains(&secondary),
                        "{} diverged {:?} {}",
                        self.lowest_root.slot,
                        p,
                        secondary
                    );
                }
            } else {
                if secondary < self.lowest_root.slot {
                    assert!(self.roots.contains(&secondary));
                }
                if primary < self.lowest_root.slot {
                    assert!(self.roots.contains(&primary));
                }
            }
        }

        let lowest_root = bank.lowest_primary_root();
        assert!(self.fork_map.get(&bank.slot).is_none());
        let mut max_root = 0;
        for n in bank.nodes.iter() {
            if n.root.slot > max_root {
                max_root = n.root.slot;
            }
        }
        self.fork_map.insert(bank.slot, bank);
        if lowest_root.slot > self.lowest_root.slot {
            let new_roots = self.compute_fork(lowest_root.slot);
            assert!(new_roots.contains(&self.lowest_root.slot));
            self.roots.extend(&new_roots);

            println!("ROOT DISTANCE {}", max_root - lowest_root.slot);
            println!(
                "LOWEST ROOT UPDATE {:?} {:?} MAX: {}",
                self.lowest_root, lowest_root, max_root
            );
            self.lowest_root = lowest_root;
            self.gc();
        }
        self.build_fork_weights();
    }

    pub fn latest_primary(&self) -> HashSet<ID> {
        self.fork_map
            .iter()
            .max_by_key(|(a, _)| *a)
            .unwrap()
            .1
            .subcom
            .primary
            .clone()
    }

    pub fn compute_fork(&self, slot: Slot) -> HashSet<Slot> {
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
        fork.into_iter().collect()
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
        //self.roots.retain(|x| x + 1000 > self.lowest_root.slot);
        self.fork_map = new_banks;
    }
    /// A validator V's vote on an ancestor X counts towards a descendant
    /// Y even if the validator is not locked out on X at Y anymore,
    /// as long as X is the latest vote observed from this validator V
    pub fn build_fork_weights(&mut self) {
        //each validators latest votes
        let mut primary_latest_votes: HashMap<ID, Slot> = HashMap::new();
        for v in self.fork_map.values() {
            v.primary_latest_votes(&mut primary_latest_votes);
        }
        //total stake voting per slot
        let mut slot_votes: HashMap<Slot, usize> = HashMap::new();
        for (_, v) in &primary_latest_votes {
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
        self.primary_fork_weights = weights;
    }
}
