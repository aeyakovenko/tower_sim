const NUM_NODES: usize = 10_000;
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
        for (i,v) in self.votes.iter_mut().enumerate() {
            if i + 1 >= self.votes.len() {
                break;
            }
            if v.lockout * 2 == self.votes[i + 1] {
                v.lockout = v.lockout * 2;
            }
        }
        let mut pop = false;
        if Some(oldest) = self.votes.back() {
            if oldest.lockout == 1<<DEPTH {
                self.root = *oldest;
                pop = true;       
            }
        }
        if pop {
            self.votes.pop_back();
        }
    }
}


struct Bank {
    nodes: [Tower, NUM_NODES];
}

impl Bank {
    fn apply(&mut self, id: usize, vote: &Vote) {
        self.nodes[id].apply(vote); 
    }
}
struct Node {
    id: usize,
    banks: HashMap<Slot, (Bank, Slot)>
}

struct Network {
    nodes: [Node; NUM_NODES];
}
