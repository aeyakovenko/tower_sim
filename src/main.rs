use tower_sim::network;

fn main() {
    let mut network = network::Network::default();
    let mut num_partitions = 0;
    let mut partition_slot = 0;
    let mut once = false;
    const TIME: usize = 128;
    for slot in 0..TIME * 2 {
        network.step();
        println!("root {:?}", network.root());
        if num_partitions == 0 && slot >= TIME && slot % TIME == 0 {
            println!("CREATING PARTITIONS===================================");
            network.create_partitions(4);
            partition_slot = slot;
            num_partitions = 4;
        }
        if num_partitions > 0 && partition_slot + TIME / 4 == slot && !once {
            println!("REPAIRING PARTITIONS=================================");
            network.repair_partitions();
            num_partitions = 0;
            once = true;
        }
    }
}
