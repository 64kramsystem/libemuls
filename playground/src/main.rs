#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

mod archive {
    pub mod _200831_clock_synchronization;
    pub mod _200831_thread_sleep_overhead;
}

fn main() {
    archive::_200831_clock_synchronization::execute();
}
