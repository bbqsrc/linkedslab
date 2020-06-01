fn main() {
    let mut map = std::collections::BTreeMap::default();

    for i in 0..100_000 {
        map.insert(i, i);
    }
}