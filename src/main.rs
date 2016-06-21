extern crate libc;

mod beeper;
mod util;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn open_battery() -> Result<(File, File), String> {
    let battery_path = Path::new("/sys/class/power_supply/BAT0");

    let mut battery_status_path = PathBuf::from(battery_path);
    battery_status_path.push("status");

    let mut battery_capacity_path = PathBuf::from(battery_path);
    battery_capacity_path.push("capacity");

    if !battery_path.exists() ||
       !battery_status_path.exists() ||
       !battery_capacity_path.exists() {
        println!("Battery path information is invalid");
        process::exit(2);
    }

    let battery_status = try!(File::open(battery_status_path)
                                .map_err(|e| e.description().to_owned()));
    let battery_capacity = try!(File::open(battery_capacity_path)
                                .map_err(|e| e.description().to_owned()));

    Ok((battery_status, battery_capacity))
}

fn setup_signal_handling(sleep_duration: Arc<Mutex<Duration>>) -> Result<thread::JoinHandle<()>, String> {
    let signal_fd = unsafe {
        let mut set: libc::sigset_t = std::mem::uninitialized();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, libc::SIGUSR1);
        if libc::pthread_sigmask(libc::SIG_BLOCK, &mut set, std::ptr::null_mut()) != 0 {
            panic!("Failed to set signal mask");
        }
        libc::signalfd(-1, &set, 0)
    };
    Ok(thread::spawn(move || {
        if signal_fd < 0 {
            panic!("Could not set up signalfd");
        }
        let mut siginfo: libc::signalfd_siginfo = unsafe { std::mem::uninitialized() };
        loop {
            let bytes = unsafe {
                libc::read(signal_fd,
                           &mut siginfo as *mut libc::signalfd_siginfo as *mut libc::c_void,
                           std::mem::size_of::<libc::signalfd_siginfo>())
            };
            if bytes != (std::mem::size_of::<libc::signalfd_siginfo>() as isize) {
                panic!("signalfd read error");
            }
            if siginfo.ssi_signo == (libc::SIGUSR1 as u32) {
                *sleep_duration.lock().unwrap() = long_sleep();
            }
        }
    }))
}

// There are no const fn constructors for Duration :(
#[inline(always)]
fn normal_sleep() -> Duration {
    Duration::from_secs(10)
}

#[inline(always)]
fn long_sleep() -> Duration {
    Duration::from_secs(600)
}

fn main() {
    let sleep_duration = Arc::new(Mutex::new(normal_sleep()));

    if !util::exists_in_path("speaker-test") {
        println!("`speaker-test` not found from PATH");
        process::exit(1);
    }
    if let Err(e) = setup_signal_handling(sleep_duration.clone()) {
        println!("{}", e);
        process::exit(4);
    }
    let (mut battery_status, mut battery_capacity) = match open_battery() {
        Ok(battery) => battery,
        Err(e) => {
            println!("{}", e);
            process::exit(3);
        }
    };

    let mut status = String::new();
    let mut capacity_str = String::new();
    loop {
        status.clear();
        capacity_str.clear();

        let _ = battery_status.read_to_string(&mut status);
        let _ = battery_capacity.read_to_string(&mut capacity_str);
        if let Err(_) = battery_status.seek(SeekFrom::Start(0)) {
            panic!("Could not seek the battery file, aborting")
        }
        if let Err(_) = battery_capacity.seek(SeekFrom::Start(0)) {
            panic!("Could not seek the battery file, aborting")
        }

        let capacity = capacity_str.trim().parse::<isize>().unwrap_or(100);

        if capacity < 10 && status.trim() == "Discharging" {
            beeper::beep();
        }

        {
            // Scope for exclusive access to sleep_duration
            let mut sleep_duration = sleep_duration.lock().unwrap();
            ::std::thread::sleep(*sleep_duration);
            *sleep_duration = normal_sleep();
        }
    }
}
