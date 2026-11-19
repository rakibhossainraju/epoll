use actix_web::{get, rt::time::sleep, web, App, HttpServer, Responder};
use std::{
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
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

async fn serve(
    host: String,
    port: u16,
    handle_signals: bool,
    ready: mpsc::Sender<io::Result<()>>,
) -> io::Result<()> {
    println!("{EXPLANATION}");
    let mut server = match HttpServer::new(|| App::new().service(delay)).bind((host, port)) {
        Ok(server) => server,
        Err(e) => {
            // io::Error isn't Clone, so report a copy and return the original.
            let _ = ready.send(Err(io::Error::new(e.kind(), e.to_string())));
            return Err(e);
        }
    };
    if !handle_signals {
        server = server.disable_signals();
    }
    // The listener is bound: connections now queue in the backlog even before
    // the server starts accepting, so it is safe to tell the caller.
    let _ = ready.send(Ok(()));
    server.run().await
}

/// Starts the server on a background thread and blocks until the listener is
/// bound, so the caller can connect immediately. Returns the bind error if the
/// address could not be bound.
/// Signals are left alone, so Ctrl+C terminates the whole process normally.
pub fn spawn(host: &str, port: u16) -> io::Result<JoinHandle<io::Result<()>>> {
    let host = host.to_string();
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        actix_web::rt::System::new().block_on(serve(host, port, false, tx))
    });
    match rx.recv() {
        Ok(Ok(())) => Ok(handle),
        Ok(Err(e)) => Err(e),
        // Sender dropped without reporting: the thread died (e.g. panicked).
        Err(_) => Err(io::Error::other("delay server thread exited before binding")),
    }
}
