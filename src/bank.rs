use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;


pub const NUM_NODES: usize = 10_000;
type ID = usize;

pub struct Bank {
    id: ID,
    nodes: [Tower; NUM_NODES],
    slot: Slot,
    parent: Slot,
    children: Vec<Slot>,
}

pub struct Block {
    slot: Slot,
    parent: Slot,
    votes: Vec<(ID, Vote)>,
}

impl Bank {
    fn child(&mut self, id: ID, slot: Slot) -> Self {
        let b = Bank {
            nodes: self.nodes.clone(),
            slot,
            parent: self.slot,
        };
        self.children.push(slot);
        b
    }
    fn apply(&mut self, block: &Block) {
        assert_eq!(self.slot, block.slot);
        assert_eq!(self.parent, block.parent);
        for (id, vote) in block.votes {
            self.nodes[id].apply(vote);
        }
    }
    fn root(&self) -> Vote {
        self.nodes[self.id].root
    }
    //check how many nodes have voted on parent and are still locked out at height
    fn latest_votes(&self, latest_votes: &mut HashMap<ID, Slot>) {
        for (i, n) in &nodes.enumerate() {
            let latest = n.latest_vote();
            let e = latest_vote.entry(i).or_insert(latest);
            if *e < latest {
                *e = latest;
            }
        }
    }
}


