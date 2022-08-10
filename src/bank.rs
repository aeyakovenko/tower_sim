use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;


pub const NUM_NODES: usize = 10_000;
type ID = usize;

pub struct Bank {
    pub id: ID,
    pub nodes: [Tower; NUM_NODES],
    pub slot: Slot,
    pub parent: Slot,
    pub children: Vec<Slot>,
}

pub struct Block {
    pub slot: Slot,
    pub parent: Slot,
    pub votes: Vec<(ID, Vote)>,
}

impl Bank {
    pub fn child(&mut self, id: ID, slot: Slot) -> Self {
        let b = Bank {
            id,
            nodes: self.nodes.clone(),
            slot,
            parent: self.slot,
            children: vec![],
        };
        self.children.push(slot);
        b
    }
    pub fn apply(&mut self, block: &Block) {
        assert_eq!(self.slot, block.slot);
        assert_eq!(self.parent, block.parent);
        for (id, vote) in &block.votes {
            self.nodes[*id].apply(vote);
        }
    }
    pub fn root(&self) -> Vote {
        self.nodes[self.id].root
    }
    //get the latest votes from each node
    pub fn latest_votes(&self, latest_votes: &mut HashMap<ID, Slot>) {
        for (i, n) in self.nodes.iter().enumerate() {
            let latest = n.latest_vote();
            let e = latest_votes.entry(i).or_insert(latest.slot);
            if *e < latest.slot {
                *e = latest.slot;
            }
        }
    }
}


