use std::thread;

use rayon::ThreadPoolBuilder;
use std::time::Duration;
// use rayon_core as rayon;

fn main() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(22)
        .build()
        .unwrap();
    // pool.spawn(op)
}
