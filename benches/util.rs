use rand::RngCore;
#[derive(Copy, Clone)]
pub struct Item([u64; 3]);

impl Item {
    pub fn new(rng: &mut rand::rngs::ThreadRng) -> Self {
        Item([rng.next_u64(), rng.next_u64(), rng.next_u64()])
    }
}
