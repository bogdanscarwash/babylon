//! `bsl-ls` — the `babylon-ls` binary. Pure stdio wire-up (plan §3: "`main.rs`
//! is a ~30-line stdio wire-up so `tests/protocol.rs` can drive the real
//! binary via `env!("CARGO_BIN_EXE_bsl-ls")`"). Every actual behavior lives
//! in the library crate (`babylon_ls::serve`) so it is testable without a
//! subprocess (`src/lifecycle.rs`'s own `Connection::memory` unit tests);
//! this file only owns process plumbing: opening the stdio transport,
//! joining its IO threads, and translating the lifecycle's return value
//! into the OS exit code.

use lsp_server::Connection;

fn main() {
    let (connection, io_threads) = Connection::stdio();

    let exit_code = babylon_ls::serve(&connection);

    // `IoThreads::join`'s writer thread blocks on its channel's own
    // `into_iter()` until every `Sender` — `connection.sender` is the only
    // one — is dropped. Drop `connection` explicitly, here, rather than
    // relying on `main`'s own end-of-scope drop: `std::process::exit`
    // below never runs destructors, so without this the writer thread
    // (and therefore `io_threads.join()`) would hang forever.
    drop(connection);

    if let Err(err) = io_threads.join() {
        eprintln!("babylon-ls: io threads failed to join: {err}");
    }

    std::process::exit(exit_code);
}
