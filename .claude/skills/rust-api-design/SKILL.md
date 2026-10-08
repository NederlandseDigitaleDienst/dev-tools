---
name: rust-api-design
description: How to shape Rust types, modules and public APIs in this repo — enums over flag-plus-Option structs, exhaustive matches, deterministic collections, newtypes, borrowing in signatures, visibility and re-exports, generics versus dyn, and rustdoc sections. Use when designing or reviewing Rust types, signatures or module layout of a Rust tool.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# Rust: shaping types and APIs

Derived from *The Rust Programming Language* by Steve Klabnik, Carol Nichols
and Chris Krycho, with contributions from the Rust Community
(https://doc.rust-lang.org/book/, MIT OR Apache-2.0); quotations are marked and
linked to their chapter.
Adapted to the Rust tools in this repository. Each tool is a crate of its own,
with its own edition and toolchain (`rust-toolchain.toml` in its folder): write
for the edition its `Cargo.toml` names. The book is written for edition 2024;
do not use 2024-only syntax in a 2021 crate, and start a new tool on 2024.
Failure handling is in the `rust-errors` skill, tests in `rust-testing`,
threads and async in `rust-concurrency`.

## Where the repo differs from the book, and why

| Difference | Status |
|---|---|
| `unwrap`/`expect`/`panic` linted outside tests | Decision (`rust-errors`) |
| Tests unwrap instead of returning `Result` | Decision (`rust-testing`) |
| thiserror in libraries instead of `Box<dyn Error>` | Decision; the book's choice is for a small application, which is what most tools are |
| Edition 2021 in `code-guide` | Taken over from the workspace it started in; a new tool starts on 2024 |

## Outdated advice to ignore

Older guides, and some skill packs, still recommend these. They no longer hold
on a current toolchain:

- `parking_lot::Mutex` over `std::sync::Mutex` for speed: std's mutex has been
  futex-based since Rust 1.62.
- `crossbeam-channel` over `std::sync::mpsc` for speed: std's channel has used
  crossbeam's implementation since Rust 1.67.
- `lazy_static!` or `once_cell`: use `std::sync::LazyLock`/`OnceLock`.
- A `G_` prefix on statics: Rust names statics and consts in
  `SCREAMING_SNAKE_CASE` with no prefix.
- async-std: discontinued; use tokio.

## Determinism

A tool must give the same output for the same input every run: an index, a
report or a page that changes order between runs cannot be compared or cached.

- **`HashMap` order is arbitrary** ([ch08-03](https://doc.rust-lang.org/book/ch08-03-hash-maps.html):
  "in an arbitrary order"). Anything that is output, serialized, hashed,
  compared or shown iterates a `BTreeMap`/`BTreeSet`, or is sorted first. A
  `HashMap` is fine for lookups whose order never escapes.
  Example of the bug: a list in a JSON response built from `.values()` of a
  `HashMap`.

## Make wrong states hard to write

- **One of several shapes is an enum with data per variant**, not a struct with
  a kind field and a set of `Option` fields of which only some combinations are
  valid ([ch06-01](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html):
  "Each variant can have different types and amounts of associated data").
- **Match your own enums exhaustively.** No `_ =>` arm and no `if let` that
  silently skips the rest on an enum the tool owns. Adding a variant must break the build at every
  place that has to decide what it means
  ([ch06-02](https://doc.rust-lang.org/book/ch06-02-match.html),
  [ch19-01](https://doc.rust-lang.org/book/ch19-01-all-the-places-for-patterns.html):
  with `if let` "the compiler doesn't check for exhaustiveness").
  `_ =>` is fine on foreign or open-ended types (strings, `serde_json::Value`).
- **Newtypes for values that must not be mixed up**: ids, paths of different
  kinds, dates as parsed values. `fn f(crate_name: &str, module: &str)` accepts
  the two in the wrong order; `fn f(krate: &CrateName, module: &ModulePath)`
  does not
  ([ch20-03](https://doc.rust-lang.org/book/ch20-03-advanced-types.html)).
  Keep the field private when the type carries an invariant (`rust-errors`).
  Do not `impl Deref` to the inner type; that hands back the API the newtype
  exists to narrow. For new APIs; not a retrofit.
- **Name your data.** A tuple of three or more in a public field, return type
  or map key becomes a struct with named fields
  ([ch05-02](https://doc.rust-lang.org/book/ch05-02-example-structs.html):
  "Tuples don't name their elements").
- **Typestate** (a distinct type per state, so a wrong transition does not
  compile, [ch18-03](https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html))
  suits in-memory builders and workflows. States persisted in a database stay
  enums.

## Signatures

- **Borrow unless the function needs ownership**; take `&str`, `&[T]`,
  `&Path`, not `&String`, `&Vec<T>`, `&PathBuf`
  ([ch04-03](https://doc.rust-lang.org/book/ch04-03-slices.html): "functions do
  not take ownership of their arguments unless they need to").
- **A `.clone()` is a visible cost** ([ch04-01](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)).
  Flag clones that only exist to get past the borrow checker, and per-request
  clones of large structures. Write `Arc::clone(&x)` / `Rc::clone(&x)` for the
  cheap pointer clone so it reads differently from a deep copy
  ([ch15-04](https://doc.rust-lang.org/book/ch15-04-rc.html)).
- **A function that consumes items takes `impl IntoIterator<Item = T>`** and
  moves them, rather than a slice it then clones from
  ([ch13-03](https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html)).
- **Generics and `impl Trait` by default; `dyn` when the types really mix at
  runtime** (a backend chosen from configuration, a heterogeneous list)
  ([ch18-02](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)). Take
  closures as `impl FnOnce`/`FnMut`/`Fn`, the loosest that works
  ([ch13-01](https://doc.rust-lang.org/book/ch13-01-closures.html)).
- **A function before a macro.** `macro_rules!` only when a function cannot do
  it (a literal for `include_str!`, variadics, generating items), with a
  comment saying which ([ch20-05](https://doc.rust-lang.org/book/ch20-05-macros.html)).
- **A type alias for repetition** (a crate `Result<T>`, a long `Arc<…>`), not
  `#[allow(clippy::type_complexity)]`. An alias adds no type safety; a newtype does.
- **Interior mutability is the exception.** `RefCell`/`Mutex` inside a value
  needs a reason the borrow checker cannot express
  ([ch15-05](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)).
  Back-references in an `Rc` graph are `Weak`
  ([ch15-06](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html)).

## Modules and visibility

- **Private by default.** `pub` only for what callers outside the module need;
  `pub(crate)` for what the crate shares internally; struct fields private when
  they carry invariants ([ch07-03](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html):
  "you know which parts of the inner code you can change without breaking the
  outer code"). This matters for the library crates; in a binary-only crate
  `pub` is crate-wide anyway.
- **Present the API with `pub use` at the crate root**, so users do not depend
  on the internal layout ([ch14-02](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html)).
- **No glob re-exports** (`pub use x::*`); they widen the public API silently.
  Glob imports only for `use super::*` in `mod tests` and well-known preludes
  ([ch07-04](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)).
- **Paths**: `crate::` by default; `super::` for a child that moves with its
  parent (`mod tests`); no `super::super`. Bring a function in through its
  parent module (`xml::tag_name(..)`), types and traits by full path.
- **File layout**: match what the crate already does (`foo/mod.rs` or `foo.rs`
  next to `foo/`), never both in one crate; a new crate uses `foo.rs`
  ([ch07-05](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)).
- **The module tree lives in `lib.rs`**; see "Binaries" in `rust-errors`.

## Derives

([appendix C](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html))
`Debug` on every public type (a manual impl when a field must not be printed,
such as a secret). `Eq` with `PartialEq` unless a field is a float or
`serde_json::Value`; `Hash` only with `Eq`; `Default` only when a zero value
means something. User-facing `Display` is written by hand (errors: thiserror's
`#[error]`).

## Docs

([ch14-02](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html))

- Every library crate and major module starts with a `//!` comment saying what
  it is for.
- A public fallible function has `# Errors`, a panicking one `# Panics`, an
  `unsafe fn` `# Safety`. Rustdoc's headings are `# Examples` (plural), not
  `# Arguments`/`# Returns` restating the signature.
- Examples are doctests, and `cargo test` runs them; non-Rust snippets are
  fenced `yaml` or `text`.

## unsafe

([ch20-01](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html))
Keep `unsafe` blocks small, behind a safe API, each with a `// SAFETY:` comment
saying how the rules are upheld. A developer tool rarely has a reason for it.

## Review checklist

- [ ] Nothing that reaches output iterates a `HashMap`
- [ ] New enums carry data per variant; matches on owned enums have no `_ =>`
- [ ] No 3+ tuples in public types or map keys; ids that can be swapped are newtypes in new APIs
- [ ] Parameters borrow (`&str`, `&[T]`); no clone just to satisfy the borrow checker
- [ ] `dyn` and macros are justified; `RefCell`/`Mutex` inside values is justified
- [ ] New items are private or `pub(crate)` unless needed outside; no `pub use x::*`
- [ ] Public types derive `Debug`; `# Errors`/`# Panics`/`# Safety` where they apply
