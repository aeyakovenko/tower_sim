use tower_sim::network;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

fn main() {
    let mut network = network::Network::default();
    let mut num_partitions = 1;
    let mut time: usize = 512;
    let mut partition_slot = 0;
    let mut repair_time = 32;
    for slot in 0..100_000 {
        network.step();
        println!("LOWEST ROOT {:?}", network.lowest_root());
        if num_partitions <= 1  && slot >= partition_slot + time && slot % time == 0 {
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
