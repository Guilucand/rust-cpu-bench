mod track_cpu;

use std::sync::{Arc, Mutex as StdMutex};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use parking_lot::Mutex;
use rayon::prelude::*;
use structopt::StructOpt;
use crate::track_cpu::set_fn;

#[derive(StructOpt)]
struct Args {
    mode: usize,
    #[structopt(short)]
    threads: Option<usize>
}

fn test_empty(cpu_count: usize) {
    println!("Test empty loop...");
    for _ in 0..cpu_count {
        std::thread::spawn(move || {
            loop {
            }
        });
    }
}

fn test_atomic_inc(cpu_count: usize) {
    println!("Test atomic inc...");
    let atomic_val = Arc::new(AtomicUsize::new(0));

    let atval = atomic_val.clone();
    set_fn(move || {
        atval.load(Ordering::SeqCst) as u128
    });

    for _ in 0..cpu_count {

        let atomic_val = atomic_val.clone();
        std::thread::spawn(move || {
            loop {
                atomic_val.fetch_add(1, Ordering::SeqCst);
            }
        });
    }
}

fn test_std_mutex(cpu_count: usize) {
    println!("Test std mutex...");
    let atomic_val = Arc::new(StdMutex::new(0));

    let atval = atomic_val.clone();
    set_fn(move || {
        *atval.lock().unwrap() as u128
    });

    for _ in 0..cpu_count {

        let atomic_val = atomic_val.clone();
        std::thread::spawn(move || {
            loop {
                *atomic_val.lock().unwrap() += 1;
            }
        });
    }
}

fn test_plot_mutex(cpu_count: usize) {
    println!("Test p.lot mutex...");
    let atomic_val = Arc::new(Mutex::new(0));

    let atval = atomic_val.clone();
    set_fn(move || {
        *atval.lock() as u128
    });

    for _ in 0..cpu_count {

        let atomic_val = atomic_val.clone();
        std::thread::spawn(move || {
            loop {
                *atomic_val.lock() += 1;
            }
        });
    }
}

fn test_uncontended_atomic(cpu_count: usize) {
    println!("Test uncontended atomic...");
    let mut atomic_vals = vec![];

    for _ in 0..cpu_count {
        atomic_vals.push(Arc::new(AtomicU64::new(0)));
    }

    for i in 0..cpu_count {
        let atomic_val = atomic_vals[i].clone();
        std::thread::spawn(move || {
            loop {
                atomic_val.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    set_fn(move || {
        let mut result = 0;

        for i in 0..cpu_count {
            result += atomic_vals[i].load(Ordering::SeqCst) as u128;
        }

        result
    });
}

#[repr(align(4096))]
struct Strided(AtomicU64);

fn test_uncontended_atomic_strided(cpu_count: usize) {
    println!("Test uncontended atomic strided...");
    let mut atomic_vals = vec![];

    for _ in 0..cpu_count {
        atomic_vals.push(Arc::new(Strided(AtomicU64::new(0))));
    }

    for i in 0..cpu_count {
        let atomic_val = atomic_vals[i].clone();
        std::thread::spawn(move || {
            loop {
                atomic_val.0.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    set_fn(move || {
        let mut result = 0;

        for i in 0..cpu_count {
            result += atomic_vals[i].0.load(Ordering::SeqCst) as u128;
        }

        result
    });
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct IntStrided(u64);

fn test_uncontended_integer_non_strided(cpu_count: usize) {
    println!("Test uncontended integer non strided...");
    static mut VALS: [u64; 1024] = [0; 1024];

    for i in 0..cpu_count {
        let refer = unsafe { &mut VALS[i] };
        std::thread::spawn(move || {
            loop {
                unsafe {
                    let val = std::ptr::read_volatile(refer as *const u64) + 1;
                    std::ptr::write_volatile(refer as *mut u64, val);
                }
            }
        });
    }

    set_fn(move || {
        let mut result = 0;

        for i in 0..cpu_count {
            result += unsafe { VALS[i] as u128 };
        }

        result
    });
}


fn test_uncontended_integer_strided(cpu_count: usize) {
    println!("Test uncontended integer non strided...");
    static mut VALS: [IntStrided; 1024] = [IntStrided(0); 1024];

    for i in 0..cpu_count {
        let refer = unsafe { &mut VALS[i].0 };
        std::thread::spawn(move || {
            loop {
                unsafe {
                    let val = std::ptr::read_volatile(refer as *const u64) + 1;
                    std::ptr::write_volatile(refer as *mut u64, val);
                }
            }
        });
    }

    set_fn(move || {
        let mut result = 0;

        for i in 0..cpu_count {
            result += unsafe { VALS[i].0 as u128 };
        }

        result
    });
}

fn test_uncontended_pmutex(cpu_count: usize) {
    println!("Test p.lot uncontended mutex...");
    let mut atomic_vals = vec![];

    for i in 0..cpu_count {
        atomic_vals.push(Arc::new(Mutex::new(0)));
    }

    for i in 0..cpu_count {

        let atomic_val = atomic_vals[i].clone();
        std::thread::spawn(move || {
            loop {
                *atomic_val.lock() += 1;
            }
        });
    }

    set_fn(move || {
        let mut result = 0;

        for i in 0..cpu_count {
            result += *atomic_vals[i].lock() as u128;
        }

        result
    });
}


fn main() {

    let args: Args = Args::from_args();

    let cpu_count = args.threads.unwrap_or(num_cpus::get());

    println!("Testing {} cpus!", cpu_count);

    track_cpu::start_tracking();

    match args.mode {
        0 => {
            test_empty(cpu_count)
        }
        1 => {
            test_atomic_inc(cpu_count)
        }
        2 => {
            test_std_mutex(cpu_count)
        }
        3 => {
            test_plot_mutex(cpu_count)
        }
        4 => {
            test_uncontended_pmutex(cpu_count)
        }
        5 => {
            test_uncontended_atomic(cpu_count)
        }
        6 => {
            test_uncontended_atomic_strided(cpu_count)
        }
        7 => {
            test_uncontended_integer_non_strided(cpu_count)
        }
        8 => {
            test_uncontended_integer_strided(cpu_count)
        }
        _ => {
            println!("Unsupported!");
            return;
        }
    }

    loop {
        thread::sleep(Duration::from_millis(1000));
    }

    println!("Hello, world!");
}
