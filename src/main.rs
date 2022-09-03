use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use tower_sim::bank::NUM_NODES;
use tower_sim::network;
use tower_sim::tower::DEPTH;

fn main() {
    //partition_test_1()
    four_partitions()
}

fn partition_test_1() {
    let mut network = network::Network::default();
    //warmup
    for _ in 0..DEPTH {
        network.step(1);
    }
    println!("LOWEST ROOT {:?}", network.lowest_root());

    //                                       /---33 - 34 -35 -36
    // 0 -> 1 -> 2 -> 3 ->... -> 31-> 32
    //                                  \ 37 - 38 -39 ... M
    //In this example you take the primary subcomittee and divide it into four groups 66, 32, 1_A, and 1_B
    let mut primary: Vec<_> = network.forks.latest_primary().into_iter().collect();
    primary.sort();
    let max = primary.len();
    let big = 2 * primary.len() / 3 - 1;
    let partitions = [
        (primary[0], primary[big]),
        (primary[big], primary[max - 2]),
        (primary[max - 2], primary[max - 1]),
        (primary[max - 1], primary[max - 1] + 1),
    ];
    println!("PARTITIONS {:?} {:?}", partitions, primary);
    println!("LOWEST ROOT {:?}", network.lowest_root());

    //1. The 1A group votes on slots 0 to 31, so its root stays 0
    let bp_66 = partitions[0].0;
    let bp_1a = partitions[3].0;
    //2. The 66  group votes 1 to 32 so makes new root at 1-4
    network.partition_step(&partitions, &[true, false, false, false], bp_66);
    network.partition_step(&partitions, &[true, false, false, false], bp_66);
    network.partition_step(&partitions, &[true, false, false, false], bp_66);
    network.partition_step(&partitions, &[true, false, false, false], bp_66);
    println!("LOWEST ROOT {:?}", network.lowest_root());
    //3. All these votes have landed in both forks

    //4. Now after the fork,  1B group starts voting on the top fork on slots 0 -> 36, so  it's rooting common ancestors 0 -> 32, updating the SMJRwhen it finally roots 1
    let bp_1b = partitions[3].0;
    network.repair_partitions(&partitions, &[true, false, false, true]);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    network.partition_step(&partitions, &[false, false, false, true], bp_1b);
    println!("LOWEST ROOT {:?}", network.lowest_root());

    //5. Meanwhile the 32 group at some point starts voting on the bottom fork, making that the heaviest fork
    let bp_32 = partitions[1].0;
    network.repair_partitions(&partitions, &[true, true, false, false]);
    for _ in 0..512 {
        network.partition_step(&partitions, &[false, true, false, false], bp_32);
        println!("LOWEST ROOT {:?}", network.lowest_root());
    }
    let root = network.lowest_root();

    //partitions reparied
    for _ in 0..512 {
        network.step(1);
        println!("LOWEST ROOT {:?}", network.lowest_root());
    }
    assert!(network.lowest_root().slot > root.slot);
}

fn four_partitions() {
    let mut network = network::Network::default();
    let mut num_partitions = 1;
    const TIME: usize = 256;
    let mut partition_slot = 0;
    for slot in 0..TIME * 100_000 {
        network.step(num_partitions);
        println!("LOWEST ROOT {:?}", network.lowest_root());
        if num_partitions == 1 && slot >= TIME && slot % TIME == 0 {
            println!("CREATING PARTITIONS===================================");
            num_partitions = 3;
            partition_slot = slot;
        }
        if num_partitions > 1 && partition_slot + TIME / 8 <= slot && slot % (TIME / 8) == 0 {
            println!("REPAIRING PARTITIONS=================================");
            num_partitions = num_partitions - 1;
        }
    }
}

fn random_partitions() {
    let mut network = network::Network::default();
    let mut num_partitions = 1;
    let mut time: usize = 512;
    let mut partition_slot = 0;
    let mut repair_time = 32;
    for slot in 0..100_000 {
        network.step(num_partitions);
        println!("LOWEST ROOT {:?}", network.lowest_root());
        if num_partitions <= 1 && slot >= partition_slot + time && slot % time == 0 {
            println!("CREATING PARTITIONS===================================");
            let mut rng = StdRng::seed_from_u64(slot as u64);
            num_partitions = rng.gen_range(2..6);
            time = rng.gen_range(16..512);
            repair_time = rng.gen_range(1..512);
            partition_slot = slot;
        }
        if num_partitions > 1 && partition_slot + repair_time <= slot && slot % repair_time == 0 {
            println!("REPAIRING PARTITIONS=================================");
            num_partitions = num_partitions - 1;
        }
    }
}
