fn main() {
    let mut map = linkedslab::SlabMap::default();

    for i in 0..100_000 {
        map.insert(i, i);
    }
}