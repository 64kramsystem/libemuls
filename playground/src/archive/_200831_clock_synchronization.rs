use rand;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc, Arc, Barrier, Condvar, Mutex,
};
use std::{thread, time::Instant};
use thread::JoinHandle;

const BENCH_RUNS: u32 = 3;

fn bench(name: &str, cycles: u32, components: u32, instructions: u32, function: fn(u32, u32, u32)) {
    let timer = Instant::now();

    for _ in 0..BENCH_RUNS {
        function(cycles, components, instructions);
    }

    let average_elapsed_time = timer.elapsed() / BENCH_RUNS;

    let million_cycles_per_second = cycles as f64
        * (1_000_000_f64 / (timer.elapsed().as_micros() / BENCH_RUNS as u128) as f64)
        / 1_000_000_f64;

    println!(
        "{:<24}: MC/S: {:.2} ({:<.2?})",
        name, million_cycles_per_second, average_elapsed_time,
    );
}

fn single_thread(cycles: u32, components: u32, instructions: u32) {
    for _ in 0..cycles {
        for _ in 0..components {
            for _ in 0..instructions {
                rand::random::<u32>();
            }
        }
    }
}

fn atomic_u32_recovery(cycles: u32, components: u32, instructions: u32) {
    let current_cycle_atom = Arc::new(AtomicU32::new(0));

    let handles = (0..components)
        .map(|_| {
            let current_cycle_atom = current_cycle_atom.clone();

            thread::spawn(move || {
                let mut last_executed_cycle = 0;

                loop {
                    let current_cycle = current_cycle_atom.load(Ordering::Relaxed);

                    for _ in last_executed_cycle..current_cycle {
                        for _ in 0..instructions {
                            rand::random::<u32>();
                        }
                    }

                    last_executed_cycle = current_cycle;

                    if current_cycle == cycles {
                        break;
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    // Either we use a signed int for last_executed_cycle, or we start at 1.
    // In a simple context like this, we do the latter.
    //
    for current_cycle in 1..(cycles + 1) {
        current_cycle_atom.store(current_cycle, Ordering::Relaxed);
    }

    for handle in handles {
        handle.join();
    }
}

// See the atomic strategy for comments.
//
fn mutex_recovery(cycles: u32, components: u32, instructions: u32) {
    let current_cycle_mutex = Arc::new(Mutex::new(0));

    let handles = (0..components)
        .map(|_| {
            let current_cycle_mutex = Arc::clone(&current_cycle_mutex);

            thread::spawn(move || {
                let mut last_executed_cycle = 0;

                loop {
                    let current_cycle = *current_cycle_mutex.lock().unwrap();

                    for _ in last_executed_cycle..current_cycle {
                        for _ in 0..instructions {
                            rand::random::<u32>();
                        }
                    }

                    last_executed_cycle = current_cycle;

                    if current_cycle == cycles {
                        break;
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for current_cycle in 1..(cycles + 1) {
        let mut current_cycle_guard = current_cycle_mutex.lock().unwrap();

        *current_cycle_guard = current_cycle;
    }

    for handle in handles {
        handle.join();
    }
}

fn bus_buffer_1(cycles: u32, components: u32, instructions: u32) {
    let mut tx = bus::Bus::new(1);

    let rxs = (0..components).map(|_| tx.add_rx());

    let handles = rxs
        .map(|rx| {
            thread::spawn(move || {
                for _ in rx {
                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for _ in 0..cycles {
        tx.broadcast(true);
    }

    std::mem::drop(tx);

    for handle in handles {
        handle.join();
    }
}

fn mpsc(cycles: u32, components: u32, instructions: u32) {
    let mut txs = vec![];

    for _ in 0..components {
        let (tx, rx) = mpsc::sync_channel(0);
        txs.push(tx);

        thread::spawn(move || {
            for _ in rx {
                for _ in 0..instructions {
                    rand::random::<u32>();
                }
            }
        });
    }

    for _ in 0..cycles {
        for tx in txs.iter() {
            tx.send(true);
        }
    }
}

fn barrier(cycles: u32, components: u32, instructions: u32) {
    let barrier = Arc::new(Barrier::new(components as usize));

    let handles = (0..components)
        .map(|_| {
            let barrier = barrier.clone();

            thread::spawn(move || {
                for _ in 0..cycles {
                    barrier.wait();

                    for _ in 0..instructions {
                        rand::random::<u32>();
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

// See the atomic strategy for comments.
//
fn condvar_recovery(cycles: u32, components: u32, instructions: u32) {
    let pair = Arc::new((Mutex::new(0), Condvar::new()));

    let handles = (0..components)
        .map(|_| {
            let pair = pair.clone();

            thread::spawn(move || {
                let mut last_executed_cycle = 0;
                let (lock, cvar) = &*pair;

                loop {
                    let current_cycle_mutex = lock.lock().unwrap();
                    let current_cycle = *current_cycle_mutex;

                    for i in last_executed_cycle..current_cycle {
                        for _ in 0..instructions {
                            rand::random::<u32>();
                        }
                    }
                    last_executed_cycle = current_cycle;

                    if current_cycle == cycles {
                        break;
                    } else {
                        cvar.wait(current_cycle_mutex).unwrap();
                    }
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    let (lock, cvar) = &*pair;

    for current_cycle in 1..(cycles + 1) {
        let mut current_cycle_mutex = lock.lock().unwrap();
        *current_cycle_mutex = current_cycle;
        cvar.notify_all();
    }

    for handle in handles {
        handle.join();
    }
}

/*
fn with_async(cycles_per_sec: u32, components: u32, instructions: u32) {
  async fn component_cycle(instructions: u32) {
      for _ in 0..instructions {
          rand::random::<u32>();
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
*/

/*

Different approaches to design synchronization of running components (essentially, a clock).
See comments in execute().

Cycles: 250000, components: 4, instructions: 15

ST                      : MC/S: 4.49 (55.69ms)
ATOMIC_U32 RECOVERY     : MC/S: 11.90 (21.01ms)
MUTEX RECOVERY          : MC/S: 12.63 (19.79ms)
BUS (BUFFER=1)          : MC/S: 0.15 (1.68s)
MPSC                    : MC/S: 0.19 (1.32s)
BARRIER                 : MC/S: 0.27 (909.42ms)

*/
pub fn execute() {
    const CYCLES: u32 = 250_000;
    const COMPONENTS: u32 = 4;
    const INSTRUCTIONS: u32 = 15;

    println!(
        "Cycles: {}, components: {}, instructions: {}\n",
        CYCLES, COMPONENTS, INSTRUCTIONS
    );

    // An optimization is to use one thread less electing one thread to clock. This doesn't improve
    // performance meanginfully, except in one case (noted).

    bench("ST", CYCLES, COMPONENTS, INSTRUCTIONS, single_thread);

    // Run with a "recovery" syncing, executing blocks of instructions since the last sync point.
    // This can't be used as is, but it gives indicative numbers about the lockless API.
    //
    bench(
        "ATOMIC_U32 RECOVERY",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        atomic_u32_recovery,
    );

    // Same as atomic u32.
    //
    bench(
        "MUTEX RECOVERY",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        mutex_recovery,
    );

    // This doesn't work, because if the last notification is sent while a thread is running, the
    // thread will go to into wait state after, and it won't wake up.
    // The performance is expected to be inferior to the mutex anyway, and due to the notifications
    // being asynchronous, they don't reallyhave any advantage.
    //
    // bench(
    //     "CONDVAR RECOVERY",
    //     CYCLES,
    //     COMPONENTS,
    //     INSTRUCTIONS,
    //     condvar_recovery,
    // );

    // Separate clock thread. The integrated version is twice as fast (!).
    // Doesn't support a synchronized channel (buffer=0), so the performance may be plain wrong.
    //
    bench(
        "BUS (BUFFER=1)",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        bus_buffer_1,
    );

    bench("MPSC", CYCLES, COMPONENTS, INSTRUCTIONS, mpsc);

    bench("BARRIER", CYCLES, COMPONENTS, INSTRUCTIONS, barrier);

    // Nonsense.
    //
    // bench("ASYNC", CYCLES_PER_SEC, COMPONENTS, INSTRUCTIONS, barrier);
}
