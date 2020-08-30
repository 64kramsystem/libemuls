extern crate test;

use futures::{executor::block_on, future::join, task, Future};
use std::{
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};
use test::bench::black_box;
use thread::JoinHandle;

const BENCH_RUNS: u32 = 1;

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

    println!("{:<24}: {:?}", name, timer.elapsed() / BENCH_RUNS);
}

fn single_thread(cycles_per_sec: u32, components: u32, instructions: u32) {
    let cycle_time = Duration::from_secs(1) / cycles_per_sec;

    for _ in 0..cycles_per_sec {
        let start_time = Instant::now();

        for _ in 0..components {
            for _ in 0..instructions {
                black_box(instructions.count_ones());
            }
        }

        let elapsed = start_time.elapsed();

        if elapsed < cycle_time {
            thread::sleep(cycle_time - elapsed);
        }
    }
}

fn barrier(cycles_per_sec: u32, components: u32, instructions: u32) {
    let cycle_time = Duration::from_secs(1) / cycles_per_sec;

    let barrier = Arc::new(Barrier::new(components as usize));

    let handles = (0..components)
        .map(|_| {
            let barrier = barrier.clone();

            thread::spawn(move || {
                for _ in 0..cycles_per_sec {
                    barrier.wait();

                    let start_time = Instant::now();

                    for _ in 0..instructions {
                        black_box(instructions.count_ones());
                    }

                    let elapsed = start_time.elapsed();

                    if elapsed < cycle_time {
                        thread::sleep(cycle_time - elapsed);
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

fn with_async(cycles_per_sec: u32, components: u32, instructions: u32) {
    async fn component_cycle(instructions: u32) {
        for _ in 0..instructions {
            black_box(instructions.count_ones());
        }
    }

    async fn components_cycle(components: u32, instructions: u32) {
        for _ in 0..components {
            component_cycle(instructions).await;
        }
    }

    async fn system_cycle(components: u32, instructions: u32) {
        // let cycle_start = Instant::now();

        block_on(components_cycle(components, instructions));

        // let elapsed = cycle_start.elapsed();

        // task::sleep(cycle_time - elapsed).await;
    }

    // futures::task::sleep(Duration::from_secs(1));

    let cycle_time = Duration::from_secs(1) / cycles_per_sec;

    block_on(system_cycle(components, instructions));
}

pub fn execute() {
    const CYCLES_PER_SEC: u32 = 30_000;
    const COMPONENTS: u32 = 4;
    const INSTRUCTIONS: u32 = 15;

    println!(
        "Cycles/sec: {}, components: {}, instructions: {}",
        CYCLES_PER_SEC, COMPONENTS, INSTRUCTIONS
    );

    if false {
        bench(
            "ST",
            CYCLES_PER_SEC,
            COMPONENTS,
            INSTRUCTIONS,
            single_thread,
        );
    }

    if false {
        bench("BARRIER", CYCLES_PER_SEC, COMPONENTS, INSTRUCTIONS, barrier);
    }
}
