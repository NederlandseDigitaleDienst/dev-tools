---
name: rust-errors
description: How Rust code in this repo fails — Result versus panic, the clippy panic lints and the panics they do not catch, error types per crate kind, fallible constructors, and how binaries report and exit. Use when writing or reviewing Rust that returns errors, can panic, adds an `#[allow(clippy::…)]`, or touches a `main.rs` or binary of a tool.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# Rust: failing well

Derived from *The Rust Programming Language* by Steve Klabnik, Carol Nichols
and Chris Krycho, with contributions from the Rust Community
(https://doc.rust-lang.org/book/, MIT OR Apache-2.0), chapters 9 and 12; quotations are marked and
linked to their chapter.
Adapted to the Rust tools in this repository. Where the book and the repo
disagree, the repo wins and the reason is given.

## The lints, and what they miss

Every Rust tool's `Cargo.toml` sets `unwrap_used`, `expect_used` and `panic` to
warn (`[lints.clippy]`), its `clippy.toml` allows all three in tests, and CI runs
clippy with `-D warnings`. The book's examples use `unwrap` and `expect` freely
("when prototyping"); never copy that into non-test code here. This is not a
disagreement with the book, which also says library code should return
`Result`: a tool that panics reports nothing useful to the developer running it
(`REVIEW.md`). Do not loosen them, and give a new Rust tool the same three.

The lints only see three spellings. These panic just the same and need the eye
of the author or reviewer:

| Panics without a lint | Use instead |
|---|---|
| `unreachable!()`, `assert!`, `todo!()` in non-test code | return an error; if it is truly a bug, see "Deliberate panics" |
| `v[i]`, `&s[a..b]` where the index can be out of range in normal operation | `.get(i)`, `first()`, `last()`, `split_first()`, then `?` or a typed error ([ch08-01](https://doc.rust-lang.org/book/ch08-01-vectors.html)) |
| `&s[a..b]` on text that may contain multi-byte characters (source code and docs do) | slice at an index from `find`/`char_indices`, or check `is_char_boundary` ([ch08-02](https://doc.rust-lang.org/book/ch08-02-strings.html)) |
| `+`, `-`, `*` on integers fed by outside input | `checked_add`/`checked_sub`/`checked_mul` mapped to an error; the book calls relying on overflow behaviour "an error" ([ch03-02](https://doc.rust-lang.org/book/ch03-02-data-types.html)) |
| `RefCell::borrow_mut()` where re-entry is possible | `try_borrow_mut()` mapped to an error ([ch15-05](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)) |

Each of these crashes the tool on input it should have reported on.

## Result or panic

Book, [ch09-03](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html):
"returning `Result` is a good default choice when you're defining a function
that might fail."

- **Expected failure returns `Result`**: bad input, a parse error, a missing
  file, a network or rate-limit error. A parser given malformed data is the
  book's own example of a case that must not panic.
- **A panic is only for a broken invariant**: a state the code itself
  guarantees can never occur, so that reaching it is a bug.

### Deliberate panics

When a failure truly cannot happen (a literal regex, a key of a fixed valid
length, a file baked in with `include_str!`):

```rust
// The pattern is a literal and is covered by a test; it cannot fail to compile.
#[allow(clippy::expect_used)]
static VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+\.\d+\.\d+$").expect("version pattern is valid"));
```

- `expect` with a message that says *why* it cannot fail, not `unwrap`. The
  book: "most Rustaceans choose `expect`… give more context about why the
  operation is expected to always succeed."
- `#[allow]` on the smallest item that needs it, never on a module or crate,
  with a comment giving the reason.
- A public function that can panic documents when, under `# Panics`, and where
  callers may hold untrusted input it gets a fallible twin (`new` panics,
  `try_new` returns `Result`).
- Do not mark a `let … else { unreachable!() }` with `// SAFETY:`. That label
  is reserved for `unsafe` blocks, so a grep for it finds exactly those.

## Validate once, in a type

Book, [ch09-03](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html):
"make a new type… put the validations in a function to create an instance of
the type rather than repeating the validations everywhere."

- A value with an invariant (an identifier, a priority in 0..=100, a parsed date)
  becomes a newtype with a **private** field, a checked constructor and a
  getter. Functions take the newtype, so they cannot be handed an unchecked
  value and do not check again.
- **serde bypasses the constructor.** A derived `Deserialize` on such a type
  accepts any value. Route it through the constructor:

  ```rust
  #[derive(Serialize, Deserialize)]
  #[serde(try_from = "i32", into = "i32")]
  pub struct Priority(i32);

  impl TryFrom<i32> for Priority {
      type Error = PriorityError;
      fn try_from(v: i32) -> Result<Self, Self::Error> { Self::try_new(v) }
  }
  impl From<Priority> for i32 {
      fn from(p: Priority) -> i32 { p.0 }
  }
  ```

  (`into` also needs the type to be `Clone`.)

  The same holds for `sqlx::FromRow`/`sqlx::Type`. (This goes past the book,
  which does not cover serde.)
- Parse a string at the boundary (HTTP, CLI, a file) into its typed form
  once, rather than passing `&str` inward and re-parsing it in several places.
- Name a fallible constructor `try_new`, `parse` or `build`; keep `new` for
  construction that cannot fail. The book renames `Config::new` to `build`
  because "many programmers expect `new` functions to never fail"
  ([ch12-03](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html)).

## Error types

- **Propagate with `?`** and `From` conversions rather than matching and
  returning by hand ([ch09-02](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)).
- **Library crates** define their errors with `thiserror` in `error.rs`, with a
  crate `Result<T>` alias.
  Not `Result<_, String>` in a public API: the caller can no longer tell one
  failure from another, and the context is whatever the string happened to say.
- **A tool's binary** may use `anyhow`, `String` errors shown to the user, or
  `Box<dyn Error>` as the
  book does at the top of an application. `Box<dyn Error>` is not `Send`; inside
  async code use `Box<dyn Error + Send + Sync>` or `anyhow`.
- **Carry the context the reader needs**: which path, which crate, which item. `map_err(|e| e.to_string())` on an I/O error yields "No such file or
  directory" with no file; `format!("writing {}: {e}", path.display())` does not.

## Binaries

Book, [ch12-03](https://doc.rust-lang.org/book/ch12-03-improving-error-handling-and-modularity.html):
main's job is limited to parsing arguments, setting up configuration, "calling
a `run` function in lib.rs" and "handling the error if `run` returns an error".

- **Thin `main.rs`.** Logic lives in the library crate, where tests reach it.
  `main.rs` `use`s the crate by name and **never re-declares its modules** with
  `mod`: that compiles every module twice, as two copies whose types do not
  match. (A binary-only tool declares its modules in `main.rs`, as `code-guide`
  does; that is the module tree, not a copy.)
- **Library code never ends the process.** No `std::process::exit` outside
  `main`; return the error (`try_from_env`, not a `from_env` that exits).
- **Errors and usage go to stderr** (`eprintln!` or `tracing`); stdout is the
  program's output ([ch12-06](https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html)).
  Exception: a tool whose stdout protocol *is* a JSON result, error included.
- **Exit non-zero on any failure**, including an unknown subcommand. Prefer
  `fn main() -> ExitCode` (or returning `Result` from `main`) over scattered
  `process::exit` calls.

## Review checklist

- [ ] No `unreachable!`/`assert!`/indexing/slicing/unchecked arithmetic/`borrow_mut`
      that input can reach in non-test code
- [ ] Every `#[allow(clippy::unwrap_used|expect_used|panic)]` is item-scoped,
      commented, and guards a failure that truly cannot happen
- [ ] Public fallible functions document `# Errors`, panicking ones `# Panics`
- [ ] Types with invariants have private fields and serde/sqlx go through the
      constructor
- [ ] No `Result<_, String>` in a library's public API; error messages name
      what failed
- [ ] `main.rs` is thin, does not `mod` the library's files, and exits non-zero on failure
