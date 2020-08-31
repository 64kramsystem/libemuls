#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
//
#![feature(test)]
#![feature(asm)]

extern crate test;

use std::sync::mpsc;
use std::thread;
use std::{
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};
use test::bench::black_box;

const BENCH_RUNS: u32 = 1;

fn bench(run: bool, name: &str, cycles: u32, cycle_instructions: u32, function: fn(u32, u32)) {
    if !run {
        return;
    }

    let timer = Instant::now();

    for _ in 0..BENCH_RUNS {
        function(cycles, cycle_instructions);
    }

    println!("{:<24}: {:?}", name, timer.elapsed() / BENCH_RUNS);
}

fn no_idling(cycles: u32, cycle_instructions: u32) {
    for _ in 0..cycles {
        for _ in 0..cycle_instructions {
            black_box(0);
        }
    }
}

fn thread_sleep(cycles: u32, cycle_instructions: u32) {
    for _ in 0..cycles {
        for _ in 0..cycle_instructions {
            black_box(0);
            thread::sleep(Duration::from_secs(0));
        }
    }
}

fn thread_sleep_1ns(cycles: u32, cycle_instructions: u32) {
    for _ in 0..cycles {
        for _ in 0..cycle_instructions {
            black_box(0);
            thread::sleep(Duration::from_nanos(1));
        }
    }
}

fn condvar(cycles: u32, cycle_instructions: u32) {
    let pair = Arc::new((Mutex::new(0), Condvar::new()));

    let handle = {
        let pair = pair.clone();

        thread::spawn(move || {
            let (lock, cvar) = &*pair;

            loop {
                let cycle_number_mutex = lock.lock().unwrap();

                for _ in 0..cycle_instructions {
                    black_box(0);
                }

                if *cycle_number_mutex == cycles - 1 {
                    break;
                } else {
                    cvar.wait(cycle_number_mutex).unwrap();
                }
            }
        })
    };

    let (lock, cvar) = &*pair;

    for cycle_number in 0..cycles {
        let mut mutex_cycle_number = lock.lock().unwrap();
        *mutex_cycle_number = cycle_number;
        cvar.notify_one();
    }

    handle.join();
}

fn noidling_channel_comm(cycles: u32, cycle_instructions: u32) {
    let (tx, rx) = mpsc::sync_channel(0);

    let handle = thread::spawn(move || loop {
        let cycle_number = rx.recv().unwrap();

        for _ in 0..cycle_instructions {
            black_box(0);
        }

        if cycle_number == cycles - 1 {
            break;
        }
    });

    for i in 1..cycles {
        tx.send(i);
    }

    handle.join();
}

// In this implementation as is, the component thread may miss cycles. This is ok, as long as
// a source component, when communicating, transmit the cycle it's at, so that they can wait
// until the destination component has reached that cycle.
//
fn noidling_atomic_incomplete(cycles: u32, cycle_instructions: u32) {
    // Convert to be 100% sure that 64 bits don't impact the performance.
    //
    let cycles = cycles as u64;

    let current_cycle = Arc::new(AtomicU64::new(0));

    let handle = {
        let current_cycle = current_cycle.clone();

        thread::spawn(move || {
            // See note in the main thread.
            //
            let mut executed_cycle = 0;

            loop {
                while executed_cycle < current_cycle.load(Ordering::Relaxed) {
                    for _ in 0..cycle_instructions {
                        black_box(0);
                    }

                    executed_cycle += 1;
                }

                if executed_cycle == cycles {
                    break;
                }
            }
        })
    };

    // Since the executed_cycle can't start from -1, for simplicity, we conventionally associate to
    // the first (executed) cycle the value 1. There's still plenty of time on a u64 though (assuming
    // 5 MHz), so i64 can actually be used - it trades off semantic for practicality.
    //
    for cycle_i in 1..(cycles + 1) {
        current_cycle.store(cycle_i, Ordering::Relaxed);
    }

    handle.join();
}

/*
NO IDLING               : 39.627234ms
THREAD SLEEP (0ns)      : 281.568917ms
THREAD SLEEP (0ns; CYCLES: 1k): 789.422546ms
CONDVAR UNSUITED        : 1.125476503s
NOIDLING/CHANNEL COMM (CYCLES: 1M): 4.453849004s
NOIDLING/ATOMIC NOCOMM  : 52.451689ms
*/
fn main() {
    const CYCLES: u32 = 10_000_000;
    const CYCLE_INSTRUCTIONS: u32 = 15;

    bench(true, "NO IDLING", CYCLES, CYCLE_INSTRUCTIONS, no_idling);

    bench(
        true,
        "THREAD SLEEP (0ns)",
        CYCLES,
        CYCLE_INSTRUCTIONS,
        thread_sleep,
    );

    bench(
        true,
        "THREAD SLEEP (0ns; CYCLES: 1k)",
        1_000,
        CYCLE_INSTRUCTIONS,
        thread_sleep_1ns,
    );

    bench(
        true,
        "CONDVAR UNSUITED",
        CYCLES,
        CYCLE_INSTRUCTIONS,
        condvar,
    );

    bench(
        true,
        "NOIDLING/CHANNEL COMM (CYCLES: 1M)",
        CYCLES / 10,
        CYCLE_INSTRUCTIONS,
        noidling_channel_comm,
    );

    // See method comment.
    bench(
        true,
        "NOIDLING/ATOMIC NOCOMM",
        CYCLES,
        CYCLE_INSTRUCTIONS,
        noidling_atomic_incomplete,
    );
}
