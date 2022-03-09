use std::os::linux::raw::stat;
use std::time::{Duration, Instant};

static mut TRACK_FN: Option<Box<dyn Fn() -> u128>> = None;

pub fn set_fn(f: impl Fn() -> u128 + 'static) {
    unsafe {
        TRACK_FN = Some(Box::new(f));
    }
}

pub fn start_tracking() {
    std::thread::spawn(|| {

        let now = Instant::now();
        let mut last_stats = simple_process_stats::ProcessStats::get().unwrap();
        let mut last_time = now.elapsed();

        loop {
            std::thread::sleep(Duration::from_millis(1000));


            let stats = simple_process_stats::ProcessStats::get().unwrap();
            let time = now.elapsed();

            let delta = time - last_time;
            let cpu_time = (stats.cpu_time_user - last_stats.cpu_time_user).as_secs_f64() / (delta.as_secs_f64());
            let sys_time = (stats.cpu_time_kernel - last_stats.cpu_time_kernel).as_secs_f64() / (delta.as_secs_f64());
            println!("Cpu usage: {:.2} System usage: {:.2} {:.2}M/s", cpu_time, sys_time,
                     unsafe {
                         TRACK_FN.as_ref().map(|f| f()).unwrap_or(0) as f64 / now.elapsed().as_secs_f64() / (1024.0 * 1024.0)
                     }
            );

            last_stats = stats;
            last_time = time;
        }
    });
}