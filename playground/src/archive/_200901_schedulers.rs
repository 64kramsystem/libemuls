use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Instant,
};
use thread::JoinHandle;

// Below this threshold (inclusive), printing is enabled.
//
const DEBUG_CYCLES_THRESHOLD: u32 = 100; // _000_000;

fn bench(name: &str, cycles: u32, components: u32, instructions: u32, function: fn(u32, u32, u32)) {
    print!("{:<40}: ", name);
    io::stdout().flush().unwrap();

    let timer = Instant::now();

    // If running more than once, discard the first run.

    function(cycles, components, instructions);

    let elapsed_time = timer.elapsed();

    let million_cycles_per_second =
        cycles as f64 * (1_000_000_f64 / elapsed_time.as_micros() as f64) / 1_000_000_f64;

    println!(
        "MC/S: {:.2} ({:<.2?})",
        million_cycles_per_second, elapsed_time,
    );
}

fn cycle_number_mutex_unsynced(cycles: u32, components: u32, instructions: u32) {
    let current_cycle_mutex = Arc::new(Mutex::new(0));
    let total_cycles = cycles * components;

    let handles = (0..components)
        .map(|_| {
            let current_cycle_mutex = current_cycle_mutex.clone();

            thread::spawn(move || loop {
                let current_cycle = {
                    let mut current_cycle_guard = current_cycle_mutex.lock().unwrap();
                    let current_cycle = *current_cycle_guard;

                    *current_cycle_guard = current_cycle + 1;

                    current_cycle
                };

                if current_cycle < total_cycles {
                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }
                } else {
                    break;
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

// See simple_cycle_number_queue_mutex for some common comments.
//
fn cycle_number_atomic_unsynced(cycles: u32, components: u32, instructions: u32) {
    let current_cycle_atomic = Arc::new(AtomicU32::new(0));
    let total_cycles = cycles * components;

    let handles = (0..components)
        .map(|_| {
            let current_cycle_atomic = current_cycle_atomic.clone();

            thread::spawn(move || loop {
                let current_cycle = current_cycle_atomic.fetch_add(1, Ordering::Relaxed);

                if current_cycle < total_cycles {
                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }
                } else {
                    break;
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

fn unclocked_assigned_mutex(cycles: u32, components: u32, instructions: u32) {
    let cycle_completion_mutex = Arc::new(Mutex::new(0));
    // This can actually be simplified to using just the components int - see round robin strategy.
    //
    let all_components_run = (1 << components) - 1;

    // Each component is assigned to a specific thread.
    //
    let handles = (0..components)
        .map(|component_i| {
            let mut cycles_completed = 0;
            let cycle_completion_mutex = cycle_completion_mutex.clone();
            let component_bitmask = 1 << component_i;

            // The mutex handling is actually not the most expensive part. If the cycle skip logic
            // is removed, and only the mutex R/W operations are left, it's a very respectable 5+
            // MC/sec. Therefore, the slow down is due to the cycles skipped due to wait. Hopefully
            // a round robin can improve this.
            //
            thread::spawn(move || {
                while cycles_completed < cycles {
                    {
                        let mut cycle_completion_guard = cycle_completion_mutex.lock().unwrap();
                        let mut cycle_completion = *cycle_completion_guard;

                        if cycle_completion & component_bitmask == 0 {
                            cycle_completion |= component_bitmask;

                            if cycle_completion == all_components_run {
                                cycle_completion = 0;
                            }

                            *cycle_completion_guard = cycle_completion;
                        } else {
                            continue;
                        }
                    };

                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }

                    cycles_completed += 1;
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

fn unclocked_round_robin_mutexes(cycles: u32, components: u32, instructions: u32) {
    let component_to_run_mutex = Arc::new(Mutex::new(components));
    let components_completed_mutex = Arc::new(Mutex::new(0));

    let handles = (0..components)
        .map(|component_i| {
            let mut cycles_completed = 0;
            let component_to_run_mutex = component_to_run_mutex.clone();
            let components_completed_mutex = components_completed_mutex.clone();

            thread::spawn(move || {
                while cycles_completed < cycles {
                    let _component_to_run = {
                        let mut component_to_run_guard = component_to_run_mutex.lock().unwrap();
                        let component_to_run = *component_to_run_guard;

                        if component_to_run == 0 {
                            let mut components_completed_guard =
                                components_completed_mutex.lock().unwrap();
                            let components_completed = *components_completed_guard;

                            if components_completed == components {
                                *component_to_run_guard = components;
                                *components_completed_guard = 0;

                                components
                            } else {
                                continue;
                            }
                        } else {
                            component_to_run
                        }
                    };

                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }

                    {
                        let mut components_completed_guard =
                            components_completed_mutex.lock().unwrap();
                        *components_completed_guard += 1;
                    }

                    cycles_completed += 1;
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

fn unclocked_round_robin_mutex_and_atomic_verify(cycles: u32, components: u32, instructions: u32) {
    let component_to_run_mutex = Arc::new(Mutex::new(components));
    let components_completed_atom = Arc::new(AtomicU32::new(0));

    let handles = (0..components)
        .map(|component_i| {
            let mut cycles_completed = 0;
            let component_to_run_mutex = component_to_run_mutex.clone();
            let components_completed_atom = components_completed_atom.clone();

            thread::spawn(move || {
                while cycles_completed < cycles {
                    let _component_to_run = {
                        let mut component_to_run_guard = component_to_run_mutex.lock().unwrap();
                        let component_to_run = *component_to_run_guard;

                        if component_to_run == 0 {
                            if components_completed_atom.load(Ordering::Relaxed) == components {
                                *component_to_run_guard = components;
                                components_completed_atom.store(0, Ordering::Relaxed);

                                components
                            } else {
                                continue;
                            }
                        } else {
                            component_to_run
                        }
                    };

                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }

                    components_completed_atom.fetch_add(1, Ordering::Relaxed);

                    cycles_completed += 1;
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

fn unclocked_round_robin_atomics(cycles: u32, components: u32, instructions: u32) {
    let components_to_run = Arc::new(AtomicI32::new(components as i32));
    let components_completed = Arc::new(AtomicU32::new(0));
    let print_debug = cycles <= DEBUG_CYCLES_THRESHOLD;

    if print_debug {
        println!();
    }

    let start_time = Instant::now();

    let handles = (0..components)
        .map(|component_i| {
            let mut cycles_completed = 0;

            let components_to_run_atom = components_to_run.clone();
            let components_completed_atom = components_completed.clone();

            thread::spawn(move || {
                while cycles_completed < cycles {
                    let components_to_run = components_to_run_atom.fetch_sub(1, Ordering::Relaxed);

                    if components_to_run > 0 {
                        for _ in 0..instructions {
                            rand::random::<u32>();
                        }

                        cycles_completed += 1;

                        let components_completed =
                            1 + components_completed_atom.fetch_add(1, Ordering::Relaxed);

                        // This sucks ~6% of performance (!).
                        //
                        // if print_debug {
                        //     println!(
                        //         "{:07}, C:{}, TR:{}, NCM:{}",
                        //         start_time.elapsed().as_nanos(),
                        //         component_i,
                        //         components_to_run,
                        //         components_completed,
                        //     );
                        // }

                        if components_completed == components {
                            // if print_debug {
                            //     println!(
                            //         "{:07}, C:{} -> RESET",
                            //         start_time.elapsed().as_nanos(),
                            //         component_i
                            //     );
                            // }

                            components_completed_atom.store(0, Ordering::Relaxed);
                            components_to_run_atom.store(components as i32, Ordering::Relaxed);
                        }
                    } else {
                        // This is great, but another performance sucker (~5%).
                        // std::sync::atomic::spin_loop_hint();

                        // if print_debug {
                        //     println!(
                        //         "{:07}, C:{}, TR:{}",
                        //         start_time.elapsed().as_nanos(),
                        //         component_i,
                        //         components_to_run
                        //     );
                        // }
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

fn unclocked_assigned_atomic_testing(cycles: u32, components: u32, instructions: u32) {
    let components_run = Arc::new(AtomicI32::new(components as i32));
    let all_components_run_bitmask = (1 << components) - 1;
    let print_debug = cycles <= DEBUG_CYCLES_THRESHOLD;

    if print_debug {
        println!();
    }

    let start_time = Instant::now();

    let handles = (0..components)
        .map(|component_i| {
            let mut cycles_completed = 0;
            let component_bitmask = 1 << component_i;

            let components_run_atom = components_run.clone();

            thread::spawn(move || {
                while cycles_completed < cycles {
                    // Preemptively set it as done; saves one atomic operation.
                    // Assumes that (write) is faster than (read + potential write).
                    //
                    while components_run_atom.fetch_or(component_bitmask, Ordering::Relaxed)
                        & component_bitmask
                        != 0
                    {
                        // Slows down performance by around 10%.
                        //
                        // std::sync::atomic::spin_loop_hint();
                    }

                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }

                    cycles_completed += 1;

                    components_run_atom.compare_and_swap(
                        all_components_run_bitmask,
                        0,
                        Ordering::Relaxed,
                    );
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

/*

First experiments with thread schedulers, possibly found a good one.

Cycles: 25.00M, components: 4, instructions: 15

CYCLE NUM MUTEX UNSYNCED (C/2)          : MC/S: 1.51 (8.30s)
CYCLE NUM ATOMIC UNSYNCED               : MC/S: 9.95 (2.51s)
UNCLOCKED ASSIGNED MUTEX (C/8)          : MC/S: 0.58 (5.36s)
UNCLOCKED ROUND ROBIN MUTEXES (C/2)     : MC/S: 1.31 (9.55s)
UNCLOCKED ROUND ROBIN MUTEX+ATOMIC VERIFY: MC/S: 4.30 (5.82s)
UNCLOCKED ROUND ROBIN ATOMICS           : MC/S: 2.91 (8.59s)
UNCLOCKED ASSIGNED ATOMIC TESTING       : MC/S: 4.46 (5.60s)

*/
pub fn execute() {
    const CYCLES: u32 = 25_000_000;
    const COMPONENTS: u32 = 4;
    const INSTRUCTIONS: u32 = 15;

    println!(
        "Cycles: {:.2}M, components: {}, instructions: {}\n",
        CYCLES as f64 / 1_000_000_f64,
        COMPONENTS,
        INSTRUCTIONS
    );

    // MPSC doesn't work here, since this is a pull architecture.

    // This is considerably slower than the atomic counterpart, while in the previous clock
    // synchronization experiments, mutex was (a bit) faster. The most obvious difference is
    // the number of writes in the mutex.
    //
    bench(
        "CYCLE NUM MUTEX UNSYNCED (C/2)",
        CYCLES / 2,
        COMPONENTS,
        INSTRUCTIONS,
        cycle_number_mutex_unsynced,
    );

    bench(
        "CYCLE NUM ATOMIC UNSYNCED",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        cycle_number_atomic_unsynced,
    );

    // "Unclocked" refers to the fact that there is no central clock pushing the cycle (number).
    // Instead, threads compute the clock number based on the number of cycles performed. A barrier
    // is implemented to signal the end of the cycle.
    //
    bench(
        "UNCLOCKED ASSIGNED MUTEX (C/8)",
        CYCLES / 8,
        COMPONENTS,
        INSTRUCTIONS,
        unclocked_assigned_mutex,
    );

    // Can be developed with one mutex, but it's slower due to more contention.
    //
    bench(
        "UNCLOCKED ROUND ROBIN MUTEXES (C/2)",
        CYCLES / 2,
        COMPONENTS,
        INSTRUCTIONS,
        unclocked_round_robin_mutexes,
    );

    // Added just for reference; may not be exact due to using an unlocked atomic invocation.
    //
    bench(
        "UNCLOCKED ROUND ROBIN MUTEX+ATOMIC VERIFY",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        unclocked_round_robin_mutex_and_atomic_verify,
    );

    // Damn! Close.
    //
    bench(
        "UNCLOCKED ROUND ROBIN ATOMICS",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        unclocked_round_robin_atomics,
    );

    bench(
        "UNCLOCKED ASSIGNED ATOMIC TESTING",
        CYCLES,
        4,
        INSTRUCTIONS,
        unclocked_assigned_atomic_testing,
    );
}
