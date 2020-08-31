use rand;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const BENCH_RUNS: u32 = 3;

fn bench(
    name: &str,
    cycles_per_sec: u32,
    components: u32,
    instructions: u32,
    function: fn(u32, u32, u32),
) {
    let timer = Instant::now();

    for _ in 0..BENCH_RUNS {
        function(cycles_per_sec, components, instructions);
    }

    println!("{:<8}: {:.2?}", name, timer.elapsed() / BENCH_RUNS);
}

fn thread_sleep(cycles_per_sec: u32, components: u32, instructions: u32) {
    let cycle_time = Duration::from_secs(1) / cycles_per_sec;

    for _ in 0..cycles_per_sec {
        let start_time = Instant::now();

        for _ in 0..components {
            for _ in 0..instructions {
                rand::random::<u32>();
            }
        }

        let elapsed = start_time.elapsed();

        if elapsed < cycle_time {
            thread::sleep(cycle_time - elapsed);
        }
    }
}

/*

Impact of sending thread to sleep between cycles.

Cycles/sec: 30000, components: 4, instructions: 15

ST      : 2.58s

*/
pub fn execute() {
    const CYCLES_PER_SEC: u32 = 30_000;
    const COMPONENTS: u32 = 4;
    const INSTRUCTIONS: u32 = 15;

    println!(
        "Cycles/sec: {}, components: {}, instructions: {}\n",
        CYCLES_PER_SEC, COMPONENTS, INSTRUCTIONS
    );

    bench(
        "SLEEP",
        CYCLES_PER_SEC,
        COMPONENTS,
        INSTRUCTIONS,
        thread_sleep,
    );
}
