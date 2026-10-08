# Developer tools

Tools for developers, made by the Nederlandse Digitale Dienst while working on
projects such as [RegelRecht](https://github.com/MinBZK/regelrecht). They are
generic: none of them depends on a particular project, and no project depends
on them.

Each tool is standalone. It lives in its own folder under `tools/`, with its
own build, dependencies, lockfile, toolchain, tests, CI workflow and releases.
A tool can be written in any language, and changing one cannot break another.

| Tool | What it does |
|------|--------------|
| [code-guide](tools/code-guide/) | A guide to a Rust workspace: its crates, modules, types and functions, and the calls between them as rust-analyzer resolves them, with an interactive call graph. |

## Using a tool

Every tool has a `README.md` with what it needs, how to install it and how to
run it, and a `justfile` with at least these recipes:

```bash
just test    # every test of the tool
just build   # build it
```

## Layout of a tool

```
tools/<name>/
  README.md     what it does, what it needs, install and use
  justfile      test, build, and whatever else the tool needs
  ...           the tool's own build files: Cargo.toml, package.json, pyproject.toml, ...
```

Each tool also has a workflow, `.github/workflows/<name>.yml`, that runs only
when files under `tools/<name>/` change. Releases are tagged per tool:
`<name>/v<version>`.

## Licence

[EUPL-1.2](LICENSE). Copyright Staat der Nederlanden (Ministerie van
Economische Zaken en Klimaat). Every file's licence is recorded
for [REUSE](https://reuse.software) in [REUSE.toml](REUSE.toml).

See [CONTRIBUTING.md](CONTRIBUTING.md) for issues and pull requests, and
[SECURITY.md](SECURITY.md) to report a vulnerability.
