const DEPTH: usize = 32;  

type Slot: u64;

struct Vote {
    slot: Slot,
    lockout: u64,
}

struct Tower {
    votes : VecDeque<(Votes; DEPTH)>,
    root: Vote,
}

impl Default for Tower {
    fn default() -> Self {
        Tower { 
            votes: VecDeque:with_capacity(DEPTH),
            root: Vote { slot: 0, lockout: 1<<DEPTH },
        }
    }
}

impl Tower {
    fn apply(&mut self, vote: &Vote) {
        assert!(vote.lockout = 2);
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
        self.votes.push_front(vote);
        for (i,v) in 1..DEPTH {
            if i >= self.votes.len() {
                break;
            }
            if v.lockout == self.votes[i - 1].lockout {
                v.lockout = v.lockout * 2;
            }
        }
        let mut root = false;
        if Some(oldest) = self.votes.back() {
            if oldest.lockout == 1<<DEPTH {
                self.root = *oldest;
                root = true;       
            }
        }
        if root {
            self.votes.pop_back();
        }
    }

    fn locked_out(&self, slot: Slot, height: Slot) -> bool {
        for v in self.votes {
            if v.slot >= slot && v.slot + v.lockout >= height {
                return true;
            }
        }
        false
    }
}


