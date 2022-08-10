use crate::tower::{Slot, Vote};
use crate::bank::{Bank, Block};
use std::collections::HashMap;

pub struct Node {
    id: ID,
    pub root: Vote,
    banks: HashMap<Slot, Bank>,
}

impl Node {
    fn apply(&mut self, block: &Block) {
        assert!(self.banks.get(&block.slot).is_none());
        let parent = self.banks.get_mut(&block.parent).unwrap();
        let mut bank = parent.child(self.id, block.slot);
        bank.apply(block);
        let root = bank.root();
        assert!(root.slot >= self.root.slot);
        if root.slot != self.root.slot {
            self.gc();
        }
        self.root = root;
        self.banks.insert(bank.slot, bank);
    }
    //only keep forks that are connected to root
    fn gc(&mut self) {
        let mut valid = vec![];
        let mut children = vec![self.root.slot];
        while !children.is_empty() {
            let slot = children.pop().unwrap();
            valid.push(slot);
            let bank = self.banks.get(&slot).unwrap();
            children.extend_from_slice(&bank.children);
        }
        let mut new_banks = HashMap::new();
        for v in valid {
            new_banks.insert(v, *banks.remove(v).unwrap());
        }
        self.banks = new_banks;
    }
    fn fork_weights(&self) -> HashMap<Slot, usize> {
        //each validators latest votes
        let mut latest_votes: HashMap<ID, Slot> = HashMap::new();
        for v in self.banks.values() {
            v.latest_votes(&mut latest_votes);
        }
        //total stake voting per slot
        let slot_votes: HashMap<Slot, usize> = HashMap::new();
        for (k, v) in &latest_votes {
            e = slot_votes.entry(v).or_insert(0);
            *e = *e + 1;
        }
        //stake weight is inherited from the parent
        let mut weights: HashMap<Slot, usize> = HashMap::new();
        let mut children = vec![self.root.slot];
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
}
