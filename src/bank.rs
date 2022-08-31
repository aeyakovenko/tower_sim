use crate::node::THRESHOLD;
use crate::subcommittee::Subcommittee;
use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;
use std::collections::HashSet;

pub const NUM_NODES: usize = 997;
pub type ID = usize;

pub struct Bank {
    pub nodes: Vec<Tower>,
    pub slot: Slot,
    pub parent: Slot,
    pub frozen: bool,
    pub children: Vec<Slot>,
    pub subcom: Subcommittee,
}

pub struct Block {
    pub slot: Slot,
    pub parent: Slot,
    pub votes: Vec<(ID, Vec<Vote>)>,
}

impl Bank {
    pub fn zero() -> Self {
        let mut nodes = vec![];
        for _ in 0..NUM_NODES {
            nodes.push(Tower::default());
        }
        Bank {
            frozen: true,
            nodes,
            slot: 0,
            parent: 0,
            subcom: Subcommittee::default(),
            children: vec![],
        }
    }
    pub fn child(&mut self, slot: Slot) -> Self {
        assert!(self.frozen);
        let mut b = Bank {
            nodes: self.nodes.clone(),
            slot,
            parent: self.slot,
            children: vec![],
            subcom: self.subcom.child(),
            frozen: false,
        };
        println!("INIT CHILD {} {}", self.slot, slot);
        let rv = b.subcom.init_child(&self.subcom);
        if rv {
            for s in &self.subcom.primary {
                assert_ne!(self.nodes[*s].root.slot, 0);
            }
        }
        self.children.push(slot);
        b
    }

    pub fn apply(&mut self, block: &Block, fork: &HashSet<Slot>) {
        assert!(!self.frozen);
        assert_eq!(self.slot, block.slot);
        assert_eq!(self.parent, block.parent);
        let min = *fork.iter().min().unwrap();
        for (id, votes) in &block.votes {
            for v in votes {
                if v.slot < min {
                    //skip votes that are too old, these are comming from a new subcommittee node
                    continue;
                }
                assert!(
                    fork.contains(&v.slot),
                    "proposed vote is not in the bank's fork {:?} {}",
                    fork,
                    v.slot
                );
                let _e = self.nodes[*id].apply(v);
            }
        }
        let primary = self.primary_super_root().slot;
        let secondary = self.secondary_super_root().slot;
        self.subcom.freeze(primary, secondary);
        self.frozen = true;
    }

    pub fn primary_calc_threshold_slot(&self, mult: u64, vote: &Vote) -> usize {
        let count: usize = self
            .subcom
            .primary
            .iter()
            .map(|p| {
                let n = &self.nodes[*p];
                //alredy rooted
                if n.root.slot >= vote.slot {
                    return 1;
                }
                for v in &n.votes {
                    if vote.lockout == 1 << THRESHOLD && v.slot >= vote.slot {
                        return 1;
                    }
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

    pub fn primary_threshold_slot(&self, vote: &Vote) -> bool {
        self.primary_calc_threshold_slot(1 << THRESHOLD, vote) > (2 * self.subcom.primary.len()) / 3
    }

    pub fn group_super_root(&self, set: &HashSet<ID>) -> Vote {
        let mut roots: Vec<_> = set.iter().map(|p| self.nodes[*p].root).collect();
        roots.sort_by_key(|x| x.slot);
        //2/3 of the nodes are at least at this root
        roots[self.subcom.primary.len() / 3]
    }

    pub fn primary_super_root(&self) -> Vote {
        self.group_super_root(&self.subcom.primary)
    }

    pub fn secondary_super_root(&self) -> Vote {
        self.group_super_root(&self.subcom.secondary)
    }

    pub fn lowest_primary_root(&self) -> Vote {
        let mut roots: Vec<_> = self
            .subcom
            .primary
            .iter()
            .map(|p| self.nodes[*p].root)
            .collect();
        roots.sort_by_key(|x| x.slot);
        roots[0]
    }

    //get the latest votes from each node
    pub fn primary_latest_votes(&self, latest_votes: &mut HashMap<ID, Slot>) {
        for p in self.subcom.primary.iter() {
            let n = &self.nodes[*p];
            let latest = n.latest_vote().unwrap_or(&n.root);
            let e = latest_votes.entry(*p).or_insert(latest.slot);
            if *e < latest.slot {
                *e = latest.slot;
            }
        }
    }
    pub fn check_primary(&self, id: ID) -> bool {
        self.subcom.primary.contains(&id)
    }
    pub fn check_subcommittee(&self, id: ID) -> bool {
        self.subcom.primary.contains(&id) || self.subcom.secondary.contains(&id)
    }
}
