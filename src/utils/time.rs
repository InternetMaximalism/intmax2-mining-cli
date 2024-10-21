pub fn sleep_until(target: u64) {
    loop {
        let now = chrono::Utc::now().timestamp() as u64;
        if now >= target {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// sleep avoiding hanging when PC is in sleep mode
pub fn sleep_for(seconds: u64) {
    let target = chrono::Utc::now().timestamp() as u64 + seconds;
    sleep_until(target);
}
