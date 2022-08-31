use crate::bank::{Bank, Block, ID, NUM_NODES};
use crate::forks::Forks;
use crate::tower::{Slot, Tower, Vote};
use std::collections::HashMap;
use std::collections::HashSet;

pub const THRESHOLD: usize = 6;

pub struct Node {
    pub id: ID,
    //local view of the bank forks
    blocks: HashSet<Slot>,
    tower: Tower,
    pub heaviest_fork: HashSet<Slot>,
}

impl Node {
    pub fn zero(id: ID) -> Self {
        let mut blocks = HashSet::new();
        blocks.insert(0);
        let mut set = HashSet::new();
        set.insert(0);
        Node {
            id,
            blocks,
            tower: Tower::default(),
            heaviest_fork: set,
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

    fn threshold_check(&self, tower: &Tower, fork_map: &HashMap<Slot, Bank>) -> bool {
        let vote = tower.votes.front().unwrap();
        let bank = fork_map.get(&vote.slot).unwrap();
        //check if the bank lockouts are increased
        let proposed_lockouts = bank.nodes[self.id].get_incrased_lockouts(1 << THRESHOLD, tower);
        if proposed_lockouts.is_empty() {
            return true;
        }
        for (slot, lockout) in proposed_lockouts {
            let v = Vote { slot, lockout };
            if !bank.primary_threshold_slot(&v) {
                if self.id < 4 {
                    println!("{} {} threshold check failed {:?}", self.id, bank.slot, v);
                }
                return false;
            }
        }
        true
    }

    fn optimistic_conf_check(
        &self,
        new_fork: &HashSet<Slot>,
        fork_weights: &HashMap<Slot, usize>,
        forks: &Forks,
    ) -> bool {
        // no votes left in tower
        if self.tower.votes.front().is_none() {
            return true;
        }
        let last_vote = self.tower.votes.front().unwrap();
        // if the last vote is a decendant of the new fork
        // no switching proof is necessary
        if new_fork.contains(&last_vote.slot) {
            return true;
        }
        //all the recent forks but those decending from the last vote must have > 1/3 votes
        let mut total = 0;
        let last_vote_fork = forks.compute_fork(last_vote.slot);
        for (slot, stake) in fork_weights {
            if self.blocks.get(slot).is_none() {
                continue;
            }
            if *slot <= last_vote.slot {
                //slot is older than last vote
                continue;
            }
            if last_vote_fork.iter().find(|x| **x == *slot).is_some() {
                //slot is a parent of the last voted fork
                continue;
            }
            let fork = forks.compute_fork(*slot);
            if fork.iter().find(|x| **x == last_vote.slot).is_none() {
                //slot is not a child of the last voted fork
                total += stake;
            }
        }
        total > NUM_NODES / 3
    }
    pub fn votes(&self) -> Vec<Vote> {
        let mut votes = self.tower.votes();
        for v in &mut votes {
            v.lockout = 2;
        }
        votes
    }
    pub fn make_block(&self, slot: Slot, votes: Vec<(ID, Vec<Vote>)>) -> Block {
        let heaviest_slot = *self.heaviest_fork.iter().max().unwrap();
        assert!(slot > heaviest_slot);
        let votes: Vec<_> = votes
            .into_iter()
            .filter(|(_, votes)| {
                votes.last().is_some()
                    && self
                        .heaviest_fork
                        .contains(&votes.last().unwrap().slot)
            })
            .collect();
        Block {
            slot,
            parent: heaviest_slot,
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

    pub fn vote(&mut self, forks: &Forks) {
        //filter out for blocks visibile to this nodes partition
        let primary_weights: HashMap<Slot, usize> = forks
            .primary_fork_weights
            .iter()
            .filter(|(x, _)| self.blocks.contains(x))
            .map(|(x, y)| (*x, *y))
            .collect();
        //compute the heaviest slot
        let heaviest_slot = primary_weights
            .iter()
            .map(|(x, y)| (y, x))
            .max()
            .map(|(_, y)| *y)
            .unwrap_or(0);
        //recursively find the fork for the heaviest slot
        let heaviest_fork = forks.compute_fork(heaviest_slot);
        assert!(heaviest_fork
            .iter()
            .find(|x| **x == forks.lowest_root.slot)
            .is_some());
        self.heaviest_fork = heaviest_fork;
        //grab the bank that this is voting on, and simulate the
        //votes applying to the forks tower state
        let bank = forks.fork_map.get(&heaviest_slot).unwrap();
        if !bank.check_subcommittee(self.id) {
            return;
        }
        //simulate the vote
        let mut tower = self.tower.clone();
        let vote = Vote {
            slot: heaviest_slot,
            lockout: 2,
        };
        //apply this vote and expire all the old votes
        if tower.apply(&vote).is_err() {
            //already voted
            return;
        }
        //check if the lockouts aren't violated
        //remaining votes in tower should be in the heaviest fork
        if !self.lockout_check(&tower) {
            if self.id < 4 {
                println!(
                    "{} recent vote is locked out from the heaviest fork {:?}",
                    self.id, tower.votes[1]
                );
            }
            return;
        }
        //compute the simulated result against the bank state
        let mut result = bank.nodes[self.id].clone();
        let proposed = tower.votes();
        assert!(proposed[0].slot <= proposed.last().unwrap().slot);
        for mut v in proposed {
            v.lockout = 2;
            let _ = result.apply(&v);
        }
        //check if the simulated result exceeds the thershold check
        //if the simulation increases the lockout, the bank should have
        //2/3+ nodes voting on the locked out slot
        if !self.threshold_check(&result, &forks.fork_map) {
            if self.id < 4 {
                println!("{} THRESHOLD CHECK FAILED", self.id);
                for (v, t) in self.tower.votes.iter().zip(result.votes.iter()) {
                    println!(
                        "{} LOCKOUT {:?} {} {:?} {}",
                        self.id,
                        v,
                        bank.primary_calc_threshold_slot(1, v),
                        t,
                        bank.primary_calc_threshold_slot(2, t)
                    );
                }
            }
            return;
        }
        //check if this node is switching forks. if its switching forks then
        //at least 1/3 of the nodes must be voting on forks that are not the last
        //vote's fork
        if !self.optimistic_conf_check(&self.heaviest_fork, &primary_weights, forks) {
            if self.id < 4 {
                println!("{} OC CHECK FAILED", self.id);
            }
            return;
        }
        if self.id < 4 {
            println!("{} voting {:?} root: {:?}", self.id, vote, self.tower.root);
        }
        for v in 1..tower.votes.len() {
            let v = &tower.votes[v];
            assert!(
                self.tower.votes.iter().find(|x| x.slot == v.slot).is_some(),
                "missing {} from {:?}",
                v.slot,
                self.tower
            );
        }
        if self.tower.root != tower.root {
            if self.id < 4 {
                println!(
                    "{} updated root {:?} old root: {:?}",
                    self.id, tower.root, self.tower.root
                );
            }
        }
        self.tower = tower;
    }
}
