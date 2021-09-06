use miniprof::*;
use std::{thread, time};

fn main() {
    let ten_millis = time::Duration::from_millis(1);
    let fifty_millis = time::Duration::from_millis(5);
    frame!();
    for _ in 0..100 {
        enter!("ten");
        thread::sleep(ten_millis);
        leave!("ten");

        enter!("fifty");
        thread::sleep(fifty_millis);
        leave!("fifty");
    }
    println!("{}", profile_data!());
}
