use tower::{Tower, Slot};

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
        Bank {
            nodes: self.nodes.clone(),
            slot,
            parent: self.slot,
        }
        self.children.push(slot);
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
    fn weight(&self, height: Slot) -> HashSet<ID> {
        let set = HashSet::new();
        for (i,n) in &nodes.enumerate() {
            if n.tower.locked_out(self.parent, height) {
                set.insert(i);
            }
        }
        set
    }
}


struct Node {
    id: ID,
    root: Vote,
    banks: HashMap<Slot, Bank>
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
        let gc = self.banks.keys().to_vec();
        for i in gc {
            if i < self.root.slot {
                self.banks.remove(i);
            }
        }
    }
    fn fork_weights(&self, height: Slot) -> HashMap<Slot, usize> {
        let weigths: HashMap<Slot, HashSet<ID>> = HashMap::new();
        //recurse through all the root children
        let mut children = vec![self.root.slot];
        while !children.is_empty() {
            let slot = children.pop();
            let bank = self.banks.get(slot).unwrap();
            children.append(bank.children.clone());
            let mut weight = bank.weight(height);
            //inherit the parent
            weight.append(weights[bank.parent].clone());
            weights.insert(slot, weight);
        }
        weights.into_iter().map(|(k,v)| (k,v.len())).collect()
    }
}

