use crate::bank::{ID, NUM_NODES};
use crate::tower::Slot;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub const SUBCOMMITTEE_EPOCH: usize = 1;
pub const SUBCOMMITTEE_SIZE: usize = 200;

pub struct Subcommittee {
    //the current primary and secondary
    pub primary: HashSet<ID>,
    pub secondary: HashSet<ID>,
    // number of times supermajority roots have increased
    // this squashes ranges of increases into 1
    pub num_super_roots: usize,
    pub parent_num_super_roots: usize,
    pub super_root: Slot,
    pub parent_super_root: Slot,
}

#[derive(PartialEq)]
pub enum Phase {
    FlipPrimary,
    SwapSecondary,
}

impl Default for Subcommittee {
    fn default() -> Self {
        let primary = Self::calc_subcommittee(0);
        let secondary = primary.clone();
        Self {
            parent_super_root: 0,
            super_root: 0,
            num_super_roots: 0,
            parent_num_super_roots: 0,
            primary,
            secondary,
        }
    }
}

pub fn hash(val: u64) -> u64 {
    let mut h = DefaultHasher::new();
    val.hash(&mut h);
    h.finish()
}

impl Subcommittee {
    pub fn child(self: &Self) -> Self {
        Self {
            parent_super_root: self.super_root,
            super_root: self.super_root,
            num_super_roots: self.num_super_roots,
            //the new subcomittee epoch is activated
            //on the child bank after the parent is frozen
            parent_num_super_roots: self.num_super_roots,
            primary: self.primary.clone(),
            secondary: self.secondary.clone(),
        }
    }
    pub fn init_child(&mut self, parent: &Self) {
        if self.epoch() != parent.epoch() {
            let epoch = self.epoch();
            match self.phase() {
                Phase::FlipPrimary => {
                    std::mem::swap(&mut self.primary, &mut self.secondary);
                    println!("FLIP PRIMARY {:?}", self.primary);
                }
                Phase::SwapSecondary => {
                    self.secondary = Self::calc_subcommittee(epoch);
                    println!("SWAP SECONDARY {:?}", self.secondary);
                }
            }
        }
    }

    pub fn freeze(&mut self, primary: Slot, secondary: Slot) {
        if self.super_root > primary {
            println!("SR {} ahead of primary {}", self.super_root, primary);
        }
        let super_root = core::cmp::min(primary, secondary);
        if self.super_root < super_root {
            self.super_root = super_root;
            if self.super_root != self.parent_super_root {
                self.num_super_roots = self.num_super_roots + 1;
                println!("NEW SR: {}", self.super_root);
            }
        }
    }

    fn calc_subcommittee(epoch: usize) -> HashSet<ID> {
        let mut set = HashSet::new();
        let mut rng = StdRng::seed_from_u64(epoch as u64);
        for _ in 0..SUBCOMMITTEE_SIZE {
            set.insert(rng.gen_range(0..NUM_NODES));
        }
        set
    }

    fn epoch(&self) -> usize {
        self.parent_num_super_roots / SUBCOMMITTEE_EPOCH
    }

    pub fn phase(&self) -> Phase {
        match self.epoch() % 2 {
            0 => Phase::FlipPrimary,
            1 => Phase::SwapSecondary,
            _ => panic!("invalid subcommittee phase"),
        }
    }
}
