# epoll

A small, from-scratch event queue built directly on Linux `epoll`, with no
`mio`, no `tokio` and no `libc` crate. It exists to show how an event loop
works underneath: how the kernel is told what to watch, how it hands ready
events back, and how a program reads from many sockets on a single thread.

The demo opens five non-blocking TCP connections to a local delay server. Each
request is delayed by a different amount (5s, 4s, 3s, 2s, 1s), so the responses
arrive in reverse order, all handled by one thread without ever blocking on a
single socket.

> This is a learning project, not a library. It is Linux-only and keeps error
> handling deliberately simple.

## Quick start

Requires Linux and a recent stable Rust toolchain (edition 2024).

```sh
cargo run      # starts the delay server, fires 5 requests, prints responses
cargo test     # runs the unit tests
cargo clippy --all-targets
```

You should see five `HTTP/1.1 200 OK` responses, arriving about one second
apart, followed by `FINISHED`.

## How it works

```
 main thread
 ┌──────────────────────────────────────────────────────────────┐
 │  client::open_requests                                       │
 │    connect ─► set_nonblocking ─► write request ─► register   │
 │                                                              │
 │  loop until every connection finished:                       │
 │    Poll::poll ──► Vec<Event> ──► Connections::handle         │
 └──────────────────────────────────────────────────────────────┘
        │ epoll_ctl (ADD)                  ▲ epoll_wait
        ▼                                  │
 ┌──────────────────────── kernel ──────────────────────────────┐
 │  interest list: fd + event mask + token                      │
 │  ready list:    entries whose fd became ready                │
 └──────────────────────────────────────────────────────────────┘
```

1. **Register.** `Registry::register` calls `epoll_ctl(EPOLL_CTL_ADD)`. The
   kernel copies an entry (the socket's fd, the event mask and a token) into its
   *interest list*. The token is opaque to the kernel and comes back unchanged
   with every event, which is how we know which connection fired.
2. **Wait.** `Poll::poll` calls `epoll_wait`, which sleeps until the *ready
   list* is non-empty. The kernel then copies the ready entries straight into
   the heap buffer of a `Vec<Event>` we pass by raw pointer. Rust cannot see
   that write, so `set_len` is used afterwards to expose the events.
3. **Dispatch.** `Connections::handle` looks up each event's token and calls
   `Connection::read_available`.
4. **Drain.** Sockets are registered edge-triggered (`EPOLLET`), so the kernel
   reports a change once. `read_available` must therefore read until
   `WouldBlock` (or EOF), or leftover data would never wake the loop again.
5. **Finish.** When a connection reaches EOF it is removed from the map.
   Dropping it closes the fd, which also removes it from the epoll set. Any
   late event for that token is ignored.

## Project layout

```
src/
├── main.rs              thin binary: constants + the event loop
├── lib.rs               declares the three modules below
├── epoll/               the OS layer
│   ├── mod.rs           private files, re-exports the public API
│   ├── ffi.rs           raw bindings: epoll_create1/ctl/wait, close, Event
│   ├── poll.rs          Poll: owns the epoll instance, wraps epoll_wait
│   └── registry.rs      Registry: wraps epoll_ctl, hands out tokens
├── net/                 per-socket state
│   ├── connection.rs    Connection: one socket, reads until WouldBlock/EOF
│   └── connections.rs   Connections: token -> Connection map, dispatches events
└── client/              the demo's request setup
    ├── connect.rs       open_requests: connect, write, register N requests
    └── http.rs          builds the HTTP GET request
delayserver/             the slow HTTP server used by the demo (own crate)
```

Each folder's `mod.rs` keeps its files private and re-exports only what other
modules need, so callers write `crate::epoll::Poll`, never
`crate::epoll::poll::Poll`.

## Design notes

- **Tokens are monotonic, never reused.** A closed connection's token is not
  handed out again, so a stale event still queued in the same batch cannot be
  applied to a newer connection. It simply finds no entry and is skipped.
- **A `HashMap` keyed by token** is used rather than a `Vec`, because finished
  connections are removed in arbitrary order.
- **The event buffer size** (`Vec::with_capacity(10)`) is the maximum number of
  events returned per `epoll_wait` call, not a limit on connections. Anything
  that does not fit stays on the kernel's ready list for the next call.
- **Errors.** A real I/O error on any socket ends the program through `?`.
  That is intentional for a demo.

## Going multithreaded

Not implemented, but the shape would be:

- Split `Poll` (one thread calls `epoll_wait`, takes `&mut self`) from a
  shareable `Registry` (`Arc`, `&self`, atomic id counter), as `mio` does.
- Insert a connection into the map **before** registering it. Otherwise an event
  can fire the instant `epoll_ctl` returns and be dropped as unknown.
- Either use one epoll instance per worker thread, with an acceptor handing out
  streams, or one poller thread feeding a worker pool. A single epoll fd polled
  by several threads needs `EPOLLONESHOT` and re-arming with `EPOLL_CTL_MOD`.

## The delay server

`delayserver` is a small HTTP server that waits for a requested duration before
replying, useful for simulating slow servers. The demo starts it on a
background thread via `delayserver::spawn("localhost", 8080)`. A request looks
like:

```
GET /[delay in ms]/[url-encoded message]
```

See [`delayserver/README.md`](delayserver/README.md) for details.

## Credits

The delay server and the overall exercise follow the epoll chapter of
*Asynchronous Programming in Rust*.
