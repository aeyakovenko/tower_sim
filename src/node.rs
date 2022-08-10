use crate::bank::{Bank, Block, ID};
use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;

pub struct Node {
    id: ID,
    pub supermajority_root: Vote,
    banks: HashMap<Slot, Bank>,
    tower: Tower,
}

impl Node {
    pub fn apply(&mut self, block: &Block) {
        assert!(self.banks.get(&block.slot).is_none());
        let parent = self.banks.get_mut(&block.parent).unwrap();
        let mut bank = parent.child(self.id, block.slot);
        bank.apply(block);
        let root = bank.supermajority_root();
        assert!(root.slot >= self.supermajority_root.slot);
        if root.slot != self.supermajority_root.slot {
            self.gc();
        }
        self.supermajority_root = root;
        self.banks.insert(bank.slot, bank);
    }
    //only keep forks that are connected to root
    pub fn gc(&mut self) {
        let mut valid = vec![];
        let mut children = vec![self.supermajority_root.slot];
        while !children.is_empty() {
            let slot = children.pop().unwrap();
            valid.push(slot);
            let bank = self.banks.get(&slot).unwrap();
            children.extend_from_slice(&bank.children);
        }
        let mut new_banks = HashMap::new();
        for v in valid {
            new_banks.insert(v, self.banks.remove(&v).unwrap());
        }
        self.banks = new_banks;
    }

    /// A validator V's vote on an ancestor X counts towards a descendant
    /// Y even if the validator is not locked out on X at Y anymore,
    /// as long as X is the latest vote observed from this validator V
    pub fn fork_weights(&self) -> HashMap<Slot, usize> {
        //each validators latest votes
        let mut latest_votes: HashMap<ID, Slot> = HashMap::new();
        for v in self.banks.values() {
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
        let mut children = vec![self.supermajority_root.slot];
        while !children.is_empty() {
            let b = children.pop().unwrap();
            let bank = self.banks.get(&b).unwrap();
            children.extend_from_slice(&bank.children);
            let parent_weight = *self
                .banks
                .get(&bank.parent)
                .map(|parent| weights.get(&parent.parent))
                .flatten()
                .unwrap_or(&0);
            let e = weights.entry(bank.parent).or_insert(parent_weight);
            *e = *e + *slot_votes.get(&bank.parent).unwrap_or(&0);
        }
        weights
    }
    pub fn vote(&mut self) -> Option<Vote> {
        let weights = self.fork_weights();
        let heaviest_slot = weights
            .iter()
            .map(|(x, y)| (y, x))
            .max()
            .map(|(_, y)| *y)
            .unwrap_or(0);
        //recursively find the fork for the heaviest slot
        let mut fork = vec![heaviest_slot];
        loop {
            if let Some(b) = self.banks.get(fork.last().unwrap()) {
                fork.push(b.parent)
            } else {
                break;
            }
        }
        let mut tower = self.tower.clone();
        let vote = Vote {
            slot: heaviest_slot,
            lockout: 2,
        };
        //apply this vote and expire all the old votes
        tower.apply(&vote);
        //the most recent unexpired vote must be in the heaviest fork
        let mut valid = true;
        if tower.votes.len() > 1 {
            valid = fork.iter().find(|x| **x == tower.votes[1].slot).is_some();
        }
        if valid {
            self.tower = tower;
            return Some(vote);
        }
        None
    }
}
