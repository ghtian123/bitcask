use std::thread;

use std::time::Duration;
use rayon::ThreadPoolBuilder;
// use rayon_core as rayon;



fn main(){
    
    let pool = rayon::ThreadPoolBuilder::new().num_threads(22).build().unwrap();
    // pool.spawn(op)

}
