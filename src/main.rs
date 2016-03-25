mod beeper;
mod util;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process;
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

fn main() {
    if !util::exists_in_path("speaker-test") {
        println!("`speaker-test` not found from PATH");
        process::exit(1);
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

        ::std::thread::sleep(Duration::from_secs(10));
    }
}
