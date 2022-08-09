const NUM_NODES: usize = 10_000;
type Slot: u64;

struct Vote {
    slot: Slot,
    lockout: u64,
}

struct Tower {
    votes : [Votes ;32],
}

struct Bank {
    nodes: [Tower, NUM_NODES];
}

struct Node {
    id: usize,
    banks: HashMap<Slot, (Bank, Slot)>
}
