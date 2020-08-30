use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc, Arc, Mutex,
};
use std::{thread, time::Instant};
use thread::JoinHandle;

const BENCH_RUNS: u32 = 3;

fn bench(
    name: &str,
    cycles: u32,
    components: u32,
    instructions: u32,
    buffer_size: usize,
    function: fn(u32, u32, u32, usize),
) {
    let timer = Instant::now();

    for _ in 0..BENCH_RUNS {
        function(cycles, components, instructions, buffer_size);
    }

    println!("{:<24}: {:?}", name, timer.elapsed() / 3);
}

fn single_thread(cycles: u32, components: u32, instructions: u32, _: usize) {
    let mut phony = 1_f64;

    for _ in 0..cycles {
        for _ in 0..components {
            for _ in 0..instructions {
                phony += phony.sqrt();
            }
        }
    }

    if phony < 1.0 {
        println!("");
    }
}

fn bus_separate_clock(cycles: u32, components: u32, instructions: u32, buffer_size: usize) {
    let mut tx = bus::Bus::new(buffer_size);

    let rxs = (0..components).map(|_| tx.add_rx());

    let handles = rxs
        .map(|rx| {
            thread::spawn(move || {
                let mut phony = 1_f64;

                for _ in rx {
                    for _ in 0..instructions {
                        phony += phony.sqrt();
                    }
                }

                if phony < 1.0 {
                    println!("");
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

fn bus_integrated_clock(cycles: u32, components: u32, instructions: u32, buffer_size: usize) {
    let mut tx: bus::Bus<bool> = bus::Bus::new(buffer_size);

    let handles = (1..components)
        .map(|_| {
            let mut rx = tx.add_rx();

            thread::spawn(move || {
                let mut phony = 1_f64;

                for _ in 0..cycles {
                    rx.recv();

                    for _ in 0..instructions {
                        phony += phony.sqrt();
                    }
                }

                if phony < 1.0 {
                    println!("");
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    let mut phony = 1_f64;

    for _ in 0..cycles {
        tx.broadcast(true);

        for _ in 0..instructions {
            phony += phony.sqrt();
        }

        if phony < 1.0 {
            println!("");
        }
    }

    for handle in handles {
        handle.join();
    }
}

fn atomic_u32(cycles: u32, components: u32, instructions: u32, _: usize) {
    let counter = Arc::new(AtomicU32::new(0));

    let handles = (0..components)
        .map(|_| {
            let counter = counter.clone();

            thread::spawn(move || {
                let mut phony = 1_f64;

                for _ in 0..cycles {
                    counter.load(Ordering::Relaxed);

                    for _ in 0..instructions {
                        phony += phony.sqrt();
                    }
                }

                if phony < 1.0 {
                    println!("");
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    loop {
        let previous_value = counter.fetch_add(1, Ordering::Relaxed);

        if previous_value == cycles - 1 {
            break;
        }
    }

    for handle in handles {
        handle.join();
    }
}

fn mutex(cycles: u32, components: u32, instructions: u32, _: usize) {
    let counter = Arc::new(Mutex::new(0));

    let handles = (0..components)
        .map(|_| {
            let counter = Arc::clone(&counter);

            thread::spawn(move || {
                let mut phony = 1_f64;

                for _ in 0..cycles {
                    counter.lock().unwrap();

                    for _ in 0..instructions {
                        phony += phony.sqrt();
                    }
                }

                if phony < 1.0 {
                    println!("");
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    loop {
        let mut counter_i = counter.lock().unwrap();
        *counter_i += 1;

        if *counter_i == cycles {
            break;
        }
    }

    for handle in handles {
        handle.join();
    }
}

fn mpsc(cycles: u32, components: u32, instructions: u32, buffer_size: usize) {
    let mut txs = vec![];

    for _ in 0..components {
        let (tx, rx) = mpsc::sync_channel(buffer_size);
        txs.push(tx);

        thread::spawn(move || {
            let mut phony = 1_f64;

            for _ in rx {
                for _ in 0..instructions {
                    phony += phony.sqrt();
                }
            }

            if phony < 1.0 {
                println!("");
            }
        });
    }

    for _ in 0..cycles {
        for tx in txs.iter() {
            tx.send(true);
        }
    }
}

fn mpsc_circular_buggy(cycles: u32, components: u32, instructions: u32, buffer_size: usize) {
    let mut txs = vec![];
    let mut rxs = vec![];

    for _ in 0..components {
        let (tx, rx) = mpsc::sync_channel(buffer_size);
        txs.push(tx);
        rxs.push(rx);
    }

    let first_receiver = rxs.remove(0);
    rxs.push(first_receiver);

    let handles = txs
        .into_iter()
        .zip(rxs)
        .map(|(tx, rx)| {
            thread::spawn(move || {
                let mut phony = 1_f64;

                for _ in 0..cycles {
                    tx.send(true);

                    for _ in 0..instructions {
                        phony += phony.sqrt();
                    }

                    rx.recv();
                }

                if phony < 1.0 {
                    println!("");
                }
            })
        })
        .collect::<Vec<JoinHandle<()>>>();

    for handle in handles {
        handle.join();
    }
}

/*
Cycles: 250000, components: 4, instructions: 15

ST 4 CMP                : 81.048419ms
ST 1 CMP                : 19.671229ms
ST 2 CMP                : 39.29752ms
ST 3 CMP                : 58.61589ms
ATOMIC_U32              : 28.261084ms
MUTEX                   : 128.130522ms
BUS SEP. CLOCK          : 1.889201513s
BUS INT. CLOCK          : 1.391656219s
MPSC                    : 1.311758936s
MPSC CIRCULAR BUG (BOUND: 1): 1.658097122s
*/
pub fn execute() {
    const CYCLES: u32 = 250_000;
    const COMPONENTS: u32 = 4;
    const INSTRUCTIONS: u32 = 15;

    println!(
        "Cycles: {}, components: {}, instructions: {}\n",
        CYCLES, COMPONENTS, INSTRUCTIONS
    );

    bench(
        "ST 4 CMP",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        0,
        single_thread,
    );

    bench("ST 1 CMP", CYCLES, 1, INSTRUCTIONS, 0, single_thread);

    bench("ST 2 CMP", CYCLES, 2, INSTRUCTIONS, 0, single_thread);

    bench("ST 3 CMP", CYCLES, 3, INSTRUCTIONS, 0, single_thread);

    // Clock integration doesn't improve speed in this specific microbenchmark.
    //
    bench(
        "ATOMIC_U32",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        0,
        atomic_u32,
    );

    // Clock integration doesn't improve speed in this specific microbenchmark.
    //
    bench("MUTEX", CYCLES, COMPONENTS, INSTRUCTIONS, 0, mutex);

    bench(
        "BUS SEP. CLOCK",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        1,
        bus_separate_clock,
    );

    bench(
        "BUS INT. CLOCK",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        1,
        bus_integrated_clock,
    );

    bench("MPSC", CYCLES, COMPONENTS, INSTRUCTIONS, 0, mpsc);

    // Has a bug (hangs with buffer=0); kept for reference as the idea of a ring is interesting.
    //
    bench(
        "MPSC CIRCULAR BUG (BOUND: 1)",
        CYCLES,
        COMPONENTS,
        INSTRUCTIONS,
        1,
        mpsc_circular_buggy,
    );
}
