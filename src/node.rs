use crate::bank::Banks;
use crate::bank::{Bank, Block, ID, NUM_NODES};
use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;
use std::collections::HashSet;

const THRESHOLD: usize = 6;

pub struct Node {
    pub id: ID,
    //local view of the bank forks
    blocks: HashSet<Slot>,
    tower: Tower,
    pub heaviest_fork: Vec<Slot>,
}

impl Node {
    pub fn zero(id: ID) -> Self {
        let mut blocks = HashSet::new();
        blocks.insert(0);
        Node {
            id,
            blocks,
            tower: Tower::default(),
            heaviest_fork: vec![0],
        }
    }

    pub fn set_active_block(&mut self, slot: Slot) {
        self.blocks.insert(slot);
        if self.blocks.len() > 1024 {
            self.gc();
        }
    }

    fn gc(&mut self) {
        self.blocks.retain(|x| *x >= self.tower.root.slot);
    }

    fn threshold_check(&self, tower: &Tower, banks: &HashMap<Slot, Bank>) -> bool {
        let proposed_lockouts =self.tower.compare_lockouts(1 << THRESHOLD, tower);
        let vote = tower.votes.front().unwrap();
        let bank = banks.get(&vote.slot).unwrap();
        for (slot, lockout) in proposed_lockouts {
            let v = Vote { slot, lockout };
            if !bank.threshold_slot(&v) {
                if self.id < 4 {
                    println!("{} threshold check failed at {:?}", self.id, v);
                }
                return false;
            }
        }
        true
    }

    fn compute_fork(&self, slot: Slot, banks: &Banks) -> Vec<Slot> {
        let mut fork = vec![slot];
        loop {
            let last = fork.last().unwrap();
            if let Some(b) = banks.fork_map.get(last) {
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

    fn optimistic_conf_check(
        &self,
        new_fork: &[Slot],
        fork_weights: &HashMap<Slot, usize>,
        banks: &Banks,
    ) -> bool {
        // no votes left in tower
        if self.tower.votes.front().is_none() {
            return true;
        }
        let last_vote = self.tower.votes.front().unwrap();
        // if the last vote is a decendant of the new fork
        // no switching proof is necessary
        if new_fork.iter().find(|x| **x == last_vote.slot).is_some() {
            return true;
        }
        //all the recent forks but those decending from the last vote must have > 1/3 votes
        let mut total = 0;
        for (slot, stake) in fork_weights {
            if self.blocks.get(slot).is_none() {
                continue;
            }
            if *slot <= last_vote.slot {
                //slot is older than last vote
                continue;
            }
            let fork = self.compute_fork(*slot, banks);
            if fork.iter().find(|x| **x == last_vote.slot).is_none() {
                //slot is not a child of the last vote
                total += stake;
            }
        }
        total > NUM_NODES / 3
    }
    pub fn last_vote(&self) -> &Vote {
        self.tower.votes.front().unwrap_or(&self.tower.root)
    }
    pub fn make_block(&self, slot: Slot, votes: Vec<(ID, Vote)>) -> Block {
        let votes: Vec<(ID, Vote)> = votes
            .into_iter()
            .filter(|(_, vote)| {
                self.heaviest_fork
                    .iter()
                    .find(|x| **x == vote.slot)
                    .is_some()
            })
            .collect();
        Block {
            slot,
            parent: *self.heaviest_fork.get(0).unwrap_or(&0),
            votes,
        }
    }

    //the latest vote in tower is from the heaviest fork
    //the second to last vote that is still live in tower
    //must be in the heaviest fork, which is the same fork
    //that generated the vote
    pub fn lockout_check(&self, tower: &Tower) -> bool {
        if tower.votes.len() > 0 {
            for e in &tower.votes {
                if self.heaviest_fork.iter().find(|x| **x == e.slot).is_none() {
                    return false;
                }
            }
            true
        } else {
            let rv = self
                .heaviest_fork
                .iter()
                .find(|x| **x == tower.root.slot)
                .is_some();
            assert!(
                rv,
                "heaviest fork doesn't contain root {} {:?}",
                tower.root.slot, self.heaviest_fork
            );
            rv
        }
    }

    pub fn vote(&mut self, banks: &Banks) {
        let weights: HashMap<Slot, usize> = banks
            .fork_weights
            .iter()
            .filter(|(x, _)| self.blocks.contains(x))
            .map(|(x, y)| (*x, *y))
            .collect();
        let heaviest_slot = weights
            .iter()
            .map(|(x, y)| (y, x))
            .max()
            .map(|(_, y)| *y)
            .unwrap_or(0);
        //recursively find the fork for the heaviest slot
        let heaviest_fork = self.compute_fork(heaviest_slot, banks);
        assert!(heaviest_fork
            .iter()
            .find(|x| **x == banks.lowest_root.slot)
            .is_some());
        self.heaviest_fork = heaviest_fork;
        if self.id < 4 {
            println!("{} heaviest fork {:?}", self.id, self.heaviest_fork);
        }
        let mut tower = self.tower.clone();
        let vote = Vote {
            slot: heaviest_slot,
            lockout: 2,
        };
        //apply this vote and expire all the old votes
        tower.apply(&vote);
        if !self.lockout_check(&tower) {
            if self.id < 4 {
                println!(
                    "{} recent vote is locked out from the heaviest fork {:?}",
                    self.id, tower.votes[1]
                );
            }
            return;
        }
        if !self.threshold_check(&tower, &banks.fork_map) {
            if self.id < 4 {
                println!("{} THRESHOLD CHECK FAILED", self.id);
                let vote = tower.votes.front().unwrap();
                let bank = banks.fork_map.get(&vote.slot).unwrap();
                for v in tower.votes.iter().rev() {
                    println!("{} LOCKOUT {:?} {}", self.id, v, bank.threshold_slot(v));
                }
            }
            return;
        }
        if !self.optimistic_conf_check(&self.heaviest_fork, &weights, banks) {
            assert!(false);
            if self.id < 4 {
                println!("{} oc check failed", self.id);
            }
            return;
        }
        if self.id < 4 {
            println!("{} voting {:?} root: {:?}", self.id, vote, self.tower.root);
        }
        self.tower = tower;
    }
}
