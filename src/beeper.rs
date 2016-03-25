use std::process::{Command, Stdio};
use std::time::Duration;

pub fn beep() {
    let mut cmd = Command::new("speaker-test")
                           .arg("-t")
                           .arg("sine")
                           .arg("-f")
                           .arg("1000")
                           .stdout(Stdio::null())
                           .stderr(Stdio::null())
                           .spawn()
                           .unwrap_or_else(|_| panic!("Could not run the beeper"));
    ::std::thread::sleep(Duration::from_millis(100));
    let _ = cmd.kill();
}
