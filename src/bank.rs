use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;

pub const NUM_NODES: usize = 1_000;
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
    pub votes: Vec<(ID, Vote)>,
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
    pub fn apply(&mut self, block: &Block) {
        assert_eq!(self.slot, block.slot);
        assert_eq!(self.parent, block.parent);
        for (id, vote) in &block.votes {
            self.nodes[*id].apply(vote);
        }
    }
    pub fn threshold_slot(&self, vote: &Vote) -> bool {
        let count: usize = self
            .nodes
            .iter()
            .map(|n| {
                for v in &n.votes {
                    //only allow lockout to be 2
                    if v.slot >= vote.slot && 2*v.lockout >= vote.lockout {
                        return 1;
                    }
                }
                0
            })
            .sum();
        count > (2 * NUM_NODES) / 3
    }
    pub fn supermajority_root(&self) -> Vote {
        let mut roots: Vec<_> = self.nodes.iter().map(|n| n.root).collect();
        roots.sort_by_key(|x| x.slot);
        //2/3 of the nodes are at least at this root
        roots[NUM_NODES / 3]
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
