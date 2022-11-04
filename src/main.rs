use crate::chain::Chain;

mod chain;

fn main() {
    let mut chain = Chain::new();
    for _ in 0..40 {
        chain.add_block();
    }
}
