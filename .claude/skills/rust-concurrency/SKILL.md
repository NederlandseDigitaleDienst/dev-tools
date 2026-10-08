---
name: rust-concurrency
description: How Rust code in this repo shares state and runs work concurrently — lock scope (and why edition 2021 makes it riskier), std versus tokio Mutex, atomics, keeping CPU and blocking work off the async runtime, closing channels, Drop, and graceful shutdown of axum services. Use when writing or reviewing Rust with threads, async/await, Mutex/RwLock/Arc, channels, `spawn_blocking`, or an axum `serve`.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# Rust: threads, async and shared state

Derived from *The Rust Programming Language* by Steve Klabnik, Carol Nichols
and Chris Krycho, with contributions from the Rust Community
(https://doc.rust-lang.org/book/, MIT OR Apache-2.0), chapters 16, 17 and 21; quotations are marked and
linked to their chapter.
Adapted to the Rust tools in this repository, which run on tokio and axum where
they serve anything. Where a rule comes from the repo or clippy rather than the
book, it says so.

## Locks

- **Hold a lock as briefly as possible.** Copy out what you need, drop the
  guard, then do the slow work.
- **Never take a lock in the scrutinee of `while let`, `if let` or `match`
  when the body does anything slow.** The temporary guard lives until the end
  of the whole block ([ch21-02](https://doc.rust-lang.org/book/ch21-02-multithreaded.html):
  "`while let` (and `if let` and `match`) does not drop temporary values until
  the end of the associated block"). Bind it with `let` first:

  ```rust
  let job = queue.lock().unwrap_or_else(PoisonError::into_inner).pop_front();
  if let Some(job) = job {
      run(job).await; // the lock is already released
  }
  ```

  On **edition 2021** (the code guide's), an `if let` scrutinee's temporaries
  also live through the `else` branch; edition 2024, which the book is written
  for, shortened that. The risk is larger in a 2021 crate than the book
  suggests. Check the tool's `Cargo.toml` for its edition.
- **`std::sync::Mutex` for short critical sections, never held across
  `.await`** (clippy's `await_holding_lock` catches the direct case).
  **`tokio::sync::Mutex` only when the lock really has to span an await**, for
  example so that concurrent callers wait for one build instead of each
  starting their own. (Repo and tokio guidance; the book does not cover the
  choice.) A check-then-act sequence (read under the lock, release, compute,
  store under the lock) is a race: two callers both compute and the slower
  one overwrites the newer result.
- **`std::sync::Mutex` and `std::sync::mpsc` are fine.** Advice to swap them for
  `parking_lot` or `crossbeam` for speed predates Rust 1.62 and 1.67.
- **A poisoned lock is recovered explicitly**
  (`unwrap_or_else(PoisonError::into_inner)`), not unwrapped; see `rust-errors`
  for the lints.
- **An atomic for a plain counter or flag; a `Mutex` when callers must wait or
  the data is compound** ([ch16-03](https://doc.rust-lang.org/book/ch16-03-shared-state.html)).
- **`Arc::clone(&state)`**, not `state.clone()`, so the cheap clone is visible
  as one.

## Where work runs

Book, [ch17-06](https://doc.rust-lang.org/book/ch17-06-futures-tasks-threads.html):
CPU-bound, parallelizable work suits threads; I/O-bound, highly concurrent work
suits async; the two combine through channels.

- **Nothing blocking or CPU-heavy on the async runtime.** Parsing a large
  index, hashing many files, `std::fs` on many or large files:
  `tokio::task::spawn_blocking`, or a dedicated thread with a channel (the
  book's pattern; `code-guide`'s server builds its model in `spawn_blocking`).
  A single small `std::fs::read_to_string` in a handler is tolerable; say so if
  you leave it.
- **Do not chop compute-bound work up with yields to make it "async"** without
  measuring; the book warns it can be "significantly slower"
  ([ch17-03](https://doc.rust-lang.org/book/ch17-03-more-futures.html)).
- **Do not repeat expensive work per request** that could be computed once and
  cached next to the shared state, especially behind an endpoint the UI polls.

## Channels

- **Move each `Sender` into its producer** (`move` closure or `async move`), so
  that when the producers finish every sender is dropped, the channel closes
  and the receiving loop ends. A sender left alive in the parent keeps the
  receiver waiting forever ([ch17-02](https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html)).

## Drop and shutdown

Book, [ch21-03](https://doc.rust-lang.org/book/ch21-03-graceful-shutdown-and-cleanup.html).

- **Shut down gracefully**: stop accepting work, let in-flight work finish,
  then exit. For an axum service that means
  `axum::serve(..).with_graceful_shutdown(signal)` listening for SIGTERM and
  ctrl-c. A plain `axum::serve(..).await` drops requests mid-flight. For a local
  developer tool that only matters when it writes something on shutdown; say so
  if you leave it out. For a worker, a `CancellationToken`.
- **`Drop` does not `unwrap`.** It can run during a panic, and a second panic
  aborts the process.
- **Do not wrap an always-present field in `Option` just so `Drop` can move it
  out**; use `mem::take`, `Vec::drain(..)` or restructure. The book calls the
  `Option` workaround a sign to "look for alternative approaches".

## Review checklist

- [ ] No guard taken in a `while let`/`if let`/`match` scrutinee with slow work in the body
- [ ] No `std::sync` guard across `.await`; any `tokio::sync::Mutex` has a reason
- [ ] No check-then-act race on shared state (lock released between check and store)
- [ ] CPU-heavy or blocking work is in `spawn_blocking` or on a thread
- [ ] Senders are moved into producers so channels close
- [ ] Services shut down gracefully; `Drop` impls do not panic
