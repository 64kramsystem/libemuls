use rand;
use std::io;
use std::io::Write;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const BENCH_RUNS: u32 = 3;

fn bench_per_sec(name: &str, cycles_per_sec: u32, instructions: u32, function: fn(u32, u32)) {
    print!("{:<24}: ", name);
    io::stdout().flush().unwrap();

    let timer = Instant::now();

    for _ in 0..BENCH_RUNS {
        function(cycles_per_sec, instructions);
    }

    println!("{:.2?}", timer.elapsed() / BENCH_RUNS);
}

// Same, but the semantics are different, and seeing a different parameter name is confusing.
//
fn bench_cycles(name: &str, cycles: u32, instructions: u32, function: fn(u32, u32)) {
    bench_per_sec(name, cycles, instructions, function);
}

fn thread_sleep(cycles_per_sec: u32, instructions: u32) {
    let cycle_time = Duration::from_secs(1) / cycles_per_sec;

    for _ in 0..cycles_per_sec {
        let start_time = Instant::now();

        for _ in 0..instructions {
            rand::random::<u32>();
        }

        let elapsed = start_time.elapsed();

        if elapsed < cycle_time {
            thread::sleep(cycle_time - elapsed);
        }
    }
}

fn futures_sleep(cycles_per_sec: u32, instructions: u32) {
    let cycle_time = Duration::from_secs(1) / cycles_per_sec;

    async fn run_cycle(instructions: u32, cycle_time: Duration) {
        for _ in 0..instructions {
            rand::random::<u32>();
        }

        // !!
        async_std::task::sleep(Duration::from_secs(1));
    }

    for i in 0..cycles_per_sec {
        let single_cycle = run_cycle(instructions, cycle_time);
        futures::executor::block_on(single_cycle);
    }
}

fn thread_nosleep(cycles_per_sec: u32, instructions: u32) {
    for _ in 0..cycles_per_sec {
        thread::spawn(move || {
            for _ in 0..instructions {
                rand::random::<u32>();
            }
        })
        .join();
    }
}

fn futures_nosleep(cycles: u32, instructions: u32) {
    async fn run_cycle(instructions: u32) {
        for _ in 0..instructions {
            rand::random::<u32>();
        }
    }

    for _ in 0..cycles {
        let single_cycle = run_cycle(instructions);
        futures::executor::block_on(single_cycle);
    }
}

fn spawn_1_thread(cycles: u32, instructions: u32) {
    async fn run_cycle(instructions: u32) {
        async_std::task::spawn(async move {
            for _ in 0..instructions {
                rand::random::<u32>();
            }
        });
    }

    for _ in 0..cycles {
        let single_cycle = run_cycle(instructions);
        futures::executor::block_on(single_cycle);
    }
}

fn spawn_4_threads(cycles: u32, instructions: u32) {
    async fn run_four_cycles(instructions: u32) {
        for i in 0..4 {
            async_std::task::spawn(async move {
                for _ in 0..instructions {
                    rand::random::<u32>();
                }
            });
        }
    }

    for _ in 0..cycles {
        let concurrent_cycles = run_four_cycles(instructions);
        futures::executor::block_on(concurrent_cycles);
    }
}

/*

Sleep impact and concurrency approaches.

Instructions: 15

Sleep:

THREAD (500C/s)         : 1.06s
THREAD (5kC/s)          : 1.25s
FUTURES NOTREAL (10M)   : 553.87ms

No sleep:

THREAD (25k)            : 1.06s
FUTURES (10M)           : 552.64ms
SPAWN (2M, 1C)          : 997.90ms
SPAWN (500k, 4C)        : 920.53ms

*/
pub fn execute() {
    const INSTRUCTIONS: u32 = 15;

    println!("Instructions: {}\n", INSTRUCTIONS);

    println!("Sleep:\n");

    bench_per_sec("THREAD (500C/s)", 500, INSTRUCTIONS, thread_sleep);

    bench_per_sec("THREAD (5kC/s)", 5_000, INSTRUCTIONS, thread_sleep);

    // Doesn't actually sleep (likely because it gets rescheduled immediately); CPU goes up to 100%.
    // No sleep also in spawn() context.
    //
    bench_cycles(
        "FUTURES NOTREAL (10M)",
        10_000_000,
        INSTRUCTIONS,
        futures_sleep,
    );

    println!("\nNo sleep:\n");

    bench_cycles("THREAD (25k)", 25_000, INSTRUCTIONS, thread_nosleep);

    bench_cycles("FUTURES (10M)", 10_000_000, INSTRUCTIONS, futures_nosleep);

    // WATCH OUT! Without `.await` invocations, spawns run straight away, so this benchmark is
    // **not** synchronized, although it's still good to know the (high) speed.
    //
    // The total execution time of the spawn approach depends only on the total futures.
    // Still, much faster than creating new threads, likely, because the executor uses a pool.

    bench_cycles("SPAWN (2M, 1C)", 2_000_000, INSTRUCTIONS, spawn_1_thread);

    bench_cycles("SPAWN (500k, 4C)", 500_000, INSTRUCTIONS, spawn_4_threads);
}
