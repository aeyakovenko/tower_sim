use tower_sim::network;

fn main() {
    let mut network = network::Network::default();
    let mut num_partitions = 1;
    const TIME: usize = 128;
    let mut partition_slot = 0;
    let mut once = false;
    for slot in 0..TIME * 1000 {
        network.step();
        println!("root {:?}", network.root());
        if !once && num_partitions == 1 && slot >= TIME && slot % TIME == 0 {
            println!("CREATING PARTITIONS===================================");
            network.create_partitions(2);
            num_partitions = 2;
            partition_slot = slot;
            once = true;
        }
        if num_partitions > 1 && partition_slot + TIME / 8 <= slot && slot % (TIME / 8) == 0 {
            println!("REPAIRING PARTITIONS=================================");
            num_partitions = num_partitions - 1;
            network.repair_partitions(num_partitions);
        }
    }
}
