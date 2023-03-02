use std::thread;
use std::time::Duration;

fn main() {
    println!("Hello, world!");
    for i in 0..100 {
        println!("hi mom!");
        thread::sleep(Duration::from_secs(1));
    }
}
