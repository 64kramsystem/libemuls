#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unused_mut)]

mod archive {
    pub mod _200831_clock_synchronization;
    pub mod _200831_sleep_and_concurrency;
    pub mod _200901_schedulers;
}

fn main() {
    archive::_200901_schedulers::execute();
}
