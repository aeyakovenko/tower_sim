use tower::{Slot, Tower};

const NUM_NODES: usize = 10_000;
type ID: usize;

struct Bank {
    id: ID,
    nodes: [Tower; NUM_NODES],
    slot: Slot,
    parent: Slot,
    children: Vec<Slot>,
}

struct Block {
    slot: Slot,
    parent: Slot,
    votes: Vec<(ID, Vote)>,
}

impl Bank {
    fn child(&mut self, slot: u64) -> Self {
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
        for v in block.votes {
            self.nodes[v.id].apply(v.vote);
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

struct Node {
    id: ID,
    root: Vote,
    banks: HashMap<Slot, Bank>,
}

impl Node {
    fn apply(&mut self, block: &Block) {
        if self.banks.get(block.slot).is_none() {
            let parent = self.banks.get_mut(block.parent).unwrap();
            let mut bank = parent.child(block.slot);
            bank.apply(block);
            let root = bank.root();
            assert!(root.slot >= self.root.slot);
            if root.slot != self.root.slot {
                self.gc();
            }
            self.root = root;
            self.banks.insert(bank.slot, bank);
        }
    }
    fn gc(&mut self) {
        let mut valid = vec![];
        let mut children = vec![self.root.slot];
        while !children.is_empty() {
            let slot = children.pop();
            valid.push(slot);
            let bank = self.banks.get(slot).unwrap();
            children.append(bank.children.clone());
        }
        let mut new_banks = HashMap::new();
        for v in valid {
            new_banks.insert(v, banks.remove(v).unwrap());
        }
        self.banks = new_banks;
    }
    fn fork_weights(&self, height: Slot) -> HashMap<Slot, usize> {
        //each validators latest votes
        let mut latest_votes: HashMap<ID, Slot> = HashMap::new();
        for v in self.banks.values() {
            v.latest_votes(&mut latest_votes);
        }
        //total stake voting per slot
        let slot_votes: HashMap<Slot, usize> = HashMap::new();
        for (k, v) in &latest_votes {
            e = slot_votes.entry(v).or_insert(0);
            *e = *e + 1;
        }
        //stake weight is inherited from the parent
        let mut weights: HashMap<Slot, u64> = HashMap::new();
        let mut children = vec![self.root.slot];
        while !children.is_empty() {
            let b = children.pop();
            let bank = self.banks.get(b);
            children.append(bank.children.clone());
            let parent_weight = self
                .banks
                .get(bank.parent)
                .flat_map(|parent| weights.get(parent.parent))
                .unwrap_or(0);
            let e = weights.entry(bank.parent).or_insert(parent_weight);
            *e = *e + slot_votes.get(bank.parent).unwrap_or(0);
        }
        weight
    }
}
