use miniprof::*;
use std::{thread, time};

fn main() {
    let alpha = time::Duration::from_micros(300);
    let bravo = time::Duration::from_micros(500);
    for _ in 0..100 {
        profiler_frame!();
        profile_scope!("mainloop");

        profiler_enter!("alpha");
        thread::sleep(alpha);
        profiler_leave!("alpha");

        profiler_enter!("bravo");
        thread::sleep(bravo);
        profiler_leave!("bravo");

        profiler_enter!("baseline");
        profiler_leave!("baseline");
    }
    println!("{}", profiler_data!());
}
