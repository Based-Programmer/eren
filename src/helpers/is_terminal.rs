pub fn is_terminal() -> bool {
    std::process::Command::new("/bin/sh")
        .args(["-c", "[ -t 0 ]"])
        .status()
        .unwrap()
        .success()
}
