mod ffi;
mod poll;

fn main() {
    // Delay server runs in the background on its own thread.
    let _server = delayserver::spawn("localhost", 8080);

    println!("Delay server started in background, main continues...");
    // ... rest of the epoll program goes here ...
    std::thread::sleep(std::time::Duration::from_secs(100));
}
