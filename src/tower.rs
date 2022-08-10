use std::collections::VecDeque;
pub const DEPTH: usize = 16;


pub type Slot = u64;

#[derive(Clone, Copy)]
pub struct Vote {
    pub slot: Slot,
    pub lockout: u64,
}

#[derive(Clone)]
pub struct Tower {
    votes: VecDeque<Vote>,
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
    pub fn apply(&mut self, vote: &Vote) {
        assert_eq!(vote.lockout, 2);
        //pop all the expired votes
        while !self.votes.is_empty() {
            let mut pop = false;
            if let Some(recent) = self.votes.front() {
                if recent.slot + recent.lockout < vote.slot {
                    pop = true;
                }
            }
            if pop {
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
    }

    pub fn latest_vote(&self) -> Vote {
        self.votes.front().unwrap_or(&self.root).clone()
    }
}
