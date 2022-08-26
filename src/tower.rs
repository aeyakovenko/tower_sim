use std::collections::HashMap;
use std::collections::VecDeque;

pub const DEPTH: usize = 16;

pub type Slot = u64;

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord, Debug)]
pub struct Vote {
    pub slot: Slot,
    pub lockout: u64,
}

impl Vote {
    pub fn new(slot: Slot) -> Self {
        Vote { slot, lockout: 2 }
    }
    pub fn zero() -> Self {
        Vote {
            slot: 0,
            lockout: 1 << DEPTH,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tower {
    pub votes: VecDeque<Vote>,
    pub root: Vote,
}

impl Default for Tower {
    fn default() -> Self {
        Tower {
            votes: VecDeque::with_capacity(DEPTH),
            root: Vote {
                slot: 0,
                lockout: 1 << DEPTH,
            },
        }
    }
}

impl Tower {
    pub fn apply(&mut self, vote: &Vote) -> Result<(), ()> {
        assert_eq!(vote.lockout, 2);
        //pop all the expired votes
        let mut expired = None;
        if self.root.slot >= vote.slot {
            return Err(());
        }
        for (i, v) in self.votes.iter().enumerate() {
            //apply only new votes
            if v.slot >= vote.slot {
                return Err(());
            }
            if v.slot + v.lockout >= vote.slot {
                break;
            }
            if v.slot + v.lockout < vote.slot {
                expired = Some(i);
            }
        }
        if let Some(i) = expired {
            for _ in 0..i {
                self.votes.pop_front();
            }
        }
        self.votes.push_front(vote.clone());
        for i in 1..DEPTH {
            if i >= self.votes.len() {
                break;
            }
            //double this lockout if the previous one is equal to this one
            if self.votes[i].lockout == self.votes[i - 1].lockout {
                self.votes[i].lockout = self.votes[i].lockout * 2;
            }
        }
        let mut root = false;
        if let Some(oldest) = self.votes.back() {
            if oldest.lockout == 1 << DEPTH {
                self.root = *oldest;
                root = true;
            }
        }
        if root {
            self.votes.pop_back();
        }
        Ok(())
    }
    //check if tower has more lockouts on a slot then in self
    pub fn get_incrased_lockouts(&self, skip_lockout: u64, tower: &Tower) -> HashMap<Slot, u64> {
        let mut rv = HashMap::new();
        let mut set = HashMap::new();
        set.insert(self.root.slot, self.root.lockout);
        for e in &self.votes {
            set.insert(e.slot, e.lockout);
        }
        if *set.get(&tower.root.slot).unwrap() < tower.root.lockout {
            rv.insert(tower.root.slot, tower.root.lockout);
        }
        for e in &tower.votes {
            if e.lockout < skip_lockout {
                continue;
            }
            let lockout = *set.get(&e.slot).unwrap_or(&u64::MAX);
            if lockout < e.lockout {
                assert_eq!(lockout * 2, e.lockout);
                rv.insert(e.slot, e.lockout);
            }
        }
        rv
    }

    pub fn votes(&self) -> Vec<Vote> {
        let mut votes = vec![self.root];
        votes.extend(self.votes.iter().rev());
        votes
    }

    pub fn latest_vote(&self) -> Option<&Vote> {
        self.votes.front()
    }
}

#[test]
fn test_compare_lockouts_1() {
    let mut t1 = Tower::default();
    let mut t2 = Tower::default();
    let v = Vote {
        slot: 1,
        lockout: 2,
    };
    assert!(!t1.compare_lockouts(0, &t2));
    t1.apply(&v);
    t2.apply(&v);
    assert!(!t1.compare_lockouts(0, &t2));
}

#[test]
fn test_compare_lockouts_2() {
    let mut t1 = Tower::default();
    let mut t2 = Tower::default();
    assert!(!t1.compare_lockouts(0, &t2));
    let v1 = Vote {
        slot: 1,
        lockout: 2,
    };
    t1.apply(&v1);
    let v2 = Vote {
        slot: 2,
        lockout: 2,
    };
    t2.apply(&v1);
    t2.apply(&v2);
    assert!(t1.compare_lockouts(0, &t2));
}

#[test]
fn test_compare_lockouts_3() {
    let mut t1 = Tower::default();
    let mut t2 = Tower::default();
    assert!(!t1.compare_lockouts(0, &t2));
    let v1 = Vote {
        slot: 1,
        lockout: 2,
    };
    let v2 = Vote {
        slot: 2,
        lockout: 2,
    };
    let v3 = Vote {
        slot: 5,
        lockout: 2,
    };

    t1.apply(&v1);
    t1.apply(&v2);
    t2.apply(&v1);
    t2.apply(&v2);
    t2.apply(&v3);
    println!("votes {:?}", t2.votes);
    println!("votes {:?}", t1.votes);
    assert!(!t1.compare_lockouts(0, &t2));
}

#[test]
fn test_apply() {
    let mut t = Tower::default();
    let v = Vote {
        slot: 1,
        lockout: 2,
    };
    t.apply(&v);
    assert_eq!(t.latest_vote(), v);
}

#[test]
fn test_root() {
    let mut t = Tower::default();
    for i in 1..(DEPTH + 1) {
        let v = Vote {
            slot: i as u64,
            lockout: 2,
        };
        t.apply(&v);
    }
    let root = Vote {
        slot: 1,
        lockout: 1 << DEPTH,
    };
    assert_eq!(t.root, root);
}

#[test]
fn test_pop_votes() {
    let mut t = Tower::default();
    for i in 1..DEPTH {
        let v = Vote {
            slot: i as u64,
            lockout: 2,
        };
        t.apply(&v);
    }
    let root = Vote {
        slot: 0,
        lockout: 1 << DEPTH,
    };
    assert_eq!(t.root, root);
    let mut test_votes: VecDeque<_> = (1..DEPTH)
        .into_iter()
        .map(|x| Vote {
            slot: DEPTH as u64 - x as u64,
            lockout: 1 << x,
        })
        .collect();
    assert_eq!(t.votes, test_votes);

    let vote = Vote {
        slot: DEPTH as u64 + 8,
        lockout: 2,
    };
    t.apply(&vote);
    assert_eq!(t.root, root);
    let _ = test_votes.pop_front();
    let _ = test_votes.pop_front();
    let _ = test_votes.pop_front();
    test_votes.push_front(vote);
    assert_eq!(t.votes, test_votes);

    let vote = Vote {
        slot: DEPTH as u64 + 9,
        lockout: 2,
    };
    t.apply(&vote);
    test_votes.push_front(vote);
    test_votes[1].lockout = 2 * test_votes[1].lockout;
    assert_eq!(t.votes, test_votes);

    let vote = Vote {
        slot: DEPTH as u64 + 10,
        lockout: 2,
    };
    t.apply(&vote);
    test_votes.push_front(vote);
    test_votes[1].lockout = 2 * test_votes[1].lockout;
    test_votes[2].lockout = 2 * test_votes[2].lockout;
    assert_eq!(t.votes, test_votes);

    let vote = Vote {
        slot: DEPTH as u64 + 11,
        lockout: 2,
    };
    t.apply(&vote);
    let root = Vote {
        slot: 1,
        lockout: 1 << DEPTH,
    };
    assert_eq!(t.root, root);
}
