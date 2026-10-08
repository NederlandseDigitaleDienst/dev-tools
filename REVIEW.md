# Review Guidelines

## Project context

This repository holds standalone developer tools. Each tool lives in
`tools/<name>/` with its own build, dependencies and tests, and runs on a
developer's machine against code it does not own: another repository, a
workspace, a checkout. A tool's errors do not reach citizens, but they do
mislead the developer who trusts its output, and a tool that writes where it
should only read can damage their work. The repository is public.

## Always check

### Correct on real input

- Does the tool handle the input real projects produce, not only the fixture?
  Generated code, macros, unusual layouts, large files, files that do not
  parse, paths with spaces or non-ASCII characters.
- When the tool cannot be sure, does it say so, rather than show a confident
  wrong answer? Leaving something out and counting it beats guessing.
- Is output deterministic: same input, same output, same order?

### Reading, not writing

- A tool writes only to its own cache or output location, never into the
  project it describes, unless writing is the point of the tool and the user
  asked for it.
- A cache is keyed so that it can never be read for the wrong input (another
  checkout, another version of the sources), and a run cut short leaves no
  cache that looks complete.

### Local servers

- A server binds to what it needs and serves only what it should: a file
  endpoint resolves paths inside the project and refuses `..`, absolute paths
  and symlinks that lead outside.

### Standalone

- A tool does not reach into another tool's folder, share its lockfile, or
  depend on a project it can describe. Shared code is copied on purpose, with
  its origin named, or becomes a tool of its own.

### Public and legal

- No secrets, private hostnames, personal data or names of private
  repositories in code, fixtures, comments or commit messages.
- A new dependency is under a licence the tool allows (`deny.toml`, the npm
  licences listed in its README). A new file type gets a line in `REUSE.toml`;
  images, fonts and logos get their own rights holder, never the code licence
  by default.
- Built output that bundles third-party assets (the design system's
  Rijkshuisstijl fonts, for one) is not committed or published without
  checking that it may be.

### Frontend

- UI is built from the NLDD design system (`nldd-*`); every custom CSS rule is
  reported and justified (AGENTS.md).

## Severity scale

- **Critical** — the tool damages the code it reads, leaks data, a security
  vulnerability, or a licence violation
- **Significant** — wrong output a developer would trust, a likely bug, a
  missing edge case on real input
- **Minor** — code quality, style, non-blocking improvement

## Skip

- Lock files (`Cargo.lock`, `package-lock.json`) unless dependencies were intentionally changed
- Generated files a check keeps in sync (`ui/src/nldd-components.js`)
- Formatting-only changes caught by `cargo fmt` or the pre-commit hooks
