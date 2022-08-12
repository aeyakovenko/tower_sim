use tower_sim::network;

fn main() {
    let mut network = network::Network::default();
    let mut num_partitions = 0;
    for slot in 0..1_000_000 {
        network.step();
        println!("root {:?}", network.root());
        if num_partitions == 0 && slot > 64 && slot <= 128 {
            println!("creating partitions");
            network.create_partitions(4);
            num_partitions = 4;
        } else if num_partitions > 0 {
            println!("repairing partitions");
            network.repair_partitions();
            num_partitions = 0;
        }
    }
}
