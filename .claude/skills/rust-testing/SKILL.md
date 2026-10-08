---
name: rust-testing
description: How to write Rust unit and integration tests in this repo — unwrap/expect instead of Result-returning tests, assertions that print what they saw, asserting the specific error, should_panic with expected, and shared helpers in tests/common. Use when writing or reviewing `#[test]`/`#[tokio::test]` code or files under a crate's `tests/` directory.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# Rust: tests

Derived from *The Rust Programming Language* by Steve Klabnik, Carol Nichols
and Chris Krycho, with contributions from the Rust Community
(https://doc.rust-lang.org/book/, MIT OR Apache-2.0), chapter 11; quotations are marked and
linked to their chapter.
Adapted to the Rust tools in this repository. How to run a tool's tests is in
its README (`just test` in its folder); this skill is about the test code.

## A failing test says why

The point of every rule here is that a red test must report what went wrong
without anyone having to rerun it under a debugger.

- **Tests unwrap; they do not return `Result`.** The book offers `-> Result`
  tests with `?` as "a convenient way" ([ch11-01](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)).
  This repo does not use them: each Rust tool's `clippy.toml` allows `unwrap`,
  `expect` and `panic` in tests, because that is how a test reports a failure,
  and a `?`-returning test hides the reason. A `?` loses the line and the value.
  Prefer `expect("what was being done")` where the step is not obvious.
  This is a deliberate choice, taken over from RegelRecht, where it followed an
  audit that found tests passing by construction; the book only calls `?`
  convenient, not better. Do not "modernize" tests towards `-> Result`.
- **Do not add `#[allow(clippy::unwrap_used)]` to a test module** in a tool
  whose `clippy.toml` already allows it.
- **An assertion prints what it saw.** `assert_eq!`/`assert_ne!` do so
  themselves. A bare `assert!` over `contains`, `matches!`, `starts_with` or a
  comparison prints only "assertion failed", so give it a message with the
  value ([ch11-01](https://doc.rust-lang.org/book/ch11-01-writing-tests.html):
  "We can see the value we actually got in the test output"):

  ```rust
  assert!(out.contains("evaluate"), "output misses the function: {out}");
  ```

## Assert the failure you mean

- **`#[should_panic(expected = "…")]`, never bare.** Without `expected` the test
  passes when the code panics for a different reason (ch11-01).
- **Check which error, not just that there was one.** The same imprecision
  applies to `assert!(r.is_err())`, which passes on any error, including the
  one from a typo in the fixture. (This extends the book's `should_panic`
  argument; the book itself shows `is_err()` as the fallback.)

  ```rust
  assert!(matches!(r, Err(IndexError::Missing(_))), "got {r:?}");
  // or
  let err = r.unwrap_err();
  assert!(err.to_string().contains("index.scip"), "got {err}");
  ```

## Does the test test what its name says?

A test whose fixture cannot produce the behaviour in its name passes whatever
the code does. Before trusting a test, ask what change to the code under test
would turn it red. If the answer is "none" (an assertion that a result is empty
on a fixture that could never produce anything), the fixture is wrong. The
`code-reviewer` skill applies the same check.

## Organization

([ch11-03](https://doc.rust-lang.org/book/ch11-03-test-organization.html))

- **Unit tests** sit in `#[cfg(test)] mod tests` in the same file, with `use
  super::*;`. That is the book's convention ("The convention is to create a
  module named `tests` in each file") and the repo's. Testing private functions
  there is fine.
- **When the test module outgrows the code, move it to its own file**:
  `#[cfg(test)] mod tests;` in `service.rs`, the tests in `service/tests.rs`.
  They stay unit tests with access to private items, and the source file stays
  readable. The book does not cover this; the standard library does it widely.
  Do it for a new or rewritten module whose tests are much larger than its
  code. Do not split existing files in passing: moving them is a change of its
  own.
- **Integration tests** in `tests/` can only reach the library crate. Logic in
  `main.rs` is out of their reach, which is one more reason to keep `main.rs`
  thin (`rust-errors`, "Binaries").
- **Helpers shared by several integration-test files go in
  `tests/common/mod.rs`**, used through `mod common;`. Not `tests/common.rs`
  (Cargo would run it as a test crate of its own), and not copied into each
  file, where the copies drift apart.
- **Doctests run under `cargo test`.** Keep examples in docs compiling.

## Review checklist

- [ ] No `-> Result` tests; no redundant `#[allow(clippy::unwrap_used)]` in tests
- [ ] Every `assert!` whose failure would not show the value has a message
- [ ] `should_panic` has `expected`; error tests match the variant or message
- [ ] The fixture can actually produce what the test name claims
- [ ] Shared integration-test helpers live in `tests/common/mod.rs`, not in copies
