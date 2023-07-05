use std::process::Command;

pub fn is_terminal() -> bool {
    Command::new("/bin/sh")
        .args(["-c", "[ -t 0 ]"])
        .status()
        .unwrap()
        .success()
}
