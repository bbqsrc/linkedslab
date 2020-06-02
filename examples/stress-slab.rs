struct Item([u64; 3]);

fn main() {
    // let mut map = linkedslab::SlabMap::default();

    println!("{:?}", std::mem::size_of::<Item>());
    println!(
        "{:?}",
        std::mem::size_of::<linkedslab::SlabMap<usize, Item>>()
    );

    // for i in 0..100_000 {
    //     map.insert(i, i);
    // }
}
