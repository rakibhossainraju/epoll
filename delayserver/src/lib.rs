use actix_web::{get, rt::time::sleep, web, App, HttpServer, Responder};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread::{self, JoinHandle},
    time::Duration,
};

pub const EXPLANATION: &str = "USAGE:
Delay server works by issuing an HTTP GET request in the format:
http://localhost:8080/[delay in ms]/[URL-encoded message]

Upon receiving a request, it immediately reports the following to the console:

{Message #} - {delay in ms}: {message}

The server then delays the response for the requested time and echoes the message back to the caller.

REQUESTS:
--------
";

static COUNTER: AtomicUsize = AtomicUsize::new(1);

#[get("/{delay}/{message}")]
async fn delay(path: web::Path<(u64, String)>) -> impl Responder {
    let (delay_ms, message) = path.into_inner();
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("#{count} - {delay_ms}ms: {message}");
    sleep(Duration::from_millis(delay_ms)).await;
    message
}

async fn serve(host: String, port: u16, handle_signals: bool) -> std::io::Result<()> {
    println!("{EXPLANATION}");
    let mut server = HttpServer::new(|| App::new().service(delay)).bind((host, port))?;
    if !handle_signals {
        server = server.disable_signals();
    }
    server.run().await
}

/// Starts the server on a background thread and returns immediately.
/// Signals are left alone, so Ctrl+C terminates the whole process normally.
pub fn spawn(host: &str, port: u16) -> JoinHandle<std::io::Result<()>> {
    let host = host.to_string();
    thread::spawn(move || actix_web::rt::System::new().block_on(serve(host, port, false)))
}
