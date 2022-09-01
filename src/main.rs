use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use tower_sim::network;
use tower_sim::tower::DEPTH;

fn main() {
    partition_test_1()
}

fn partition_test_1() {
    let mut network = network::Network::default();
    //warmup
    for _ in 0..(DEPTH*2) {
        network.step();
    }

    //                                       /---33 - 34 -35 -36
    // 0 -> 1 -> 2 -> 3 ->... -> 31-> 32
    //                                  \ 37 - 38 -39 ... M
    //In this example you take the primary subcomittee and divide it into four groups 66, 32, 1_A, and 1_B
    let partitions = [666, 332, 1, 1];

    //1. The 1A group votes on slots 0 to 31, so its root stays 0 
    network.step(&partitions, &[2]);
    //2. The 66  group votes 1 to 32 so makes new root at 1
    for _ in 0..DEPTH - 2 {
        network.partition_step(&partitions, &[0,2]);
    }
    network.partition_step(&partitions, &[0]);
    //3. All these votes have landed in both forks


    //4. Now after the fork,  1B group starts voting on the top fork on slots 0 -> 36, so  it's rooting common ancestors 0 -> 32, updating the SMJRwhen it finally roots 1
    network.partition_step(&partitions, &[3]);
    network.partition_step(&partitions, &[3]);
    network.partition_step(&partitions, &[3]);
    network.partition_step(&partitions, &[3]);

    //5. Meanwhile the 32 group at some point starts voting on the bottom fork, making that the heaviest fork
    for _ in 0..512 {
        network.partition_step(&partitions, &[2]);
    }

    //partitions reparied 
    for _ in 0..512 {
        network.step();
    }
}

fn four_partitions() {
    let mut network = network::Network::default();
    let mut num_partitions = 1;
    const TIME: usize = 256;
    let mut partition_slot = 0;
    for slot in 0..TIME * 100_000 {
        network.step();
        println!("LOWEST ROOT {:?}", network.lowest_root());
        if num_partitions == 1 && slot >= TIME && slot % TIME == 0 {
            println!("CREATING PARTITIONS===================================");
            network.create_partitions(4);
            num_partitions = 4;
            partition_slot = slot;
        }
        if num_partitions > 1 && partition_slot + TIME / 8 <= slot && slot % (TIME / 8) == 0 {
            println!("REPAIRING PARTITIONS=================================");
            num_partitions = num_partitions - 1;
            network.repair_partitions(num_partitions);
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
        network.step();
        println!("LOWEST ROOT {:?}", network.lowest_root());
        if num_partitions <= 1 && slot >= partition_slot + time && slot % time == 0 {
            println!("CREATING PARTITIONS===================================");
            let mut rng = StdRng::seed_from_u64(slot as u64);
            num_partitions = rng.gen_range(2..6);
            time = rng.gen_range(16..512);
            repair_time = rng.gen_range(1..512);
            network.create_partitions(num_partitions);
            partition_slot = slot;
        }
        if num_partitions > 1 && partition_slot + repair_time <= slot && slot % repair_time == 0 {
            println!("REPAIRING PARTITIONS=================================");
            num_partitions = num_partitions - 1;
            network.repair_partitions(num_partitions);
        }
    }
}
