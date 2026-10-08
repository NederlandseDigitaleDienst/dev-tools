# AGENTS.md

Instructions for coding agents working in this repository (Codex, Copilot,
Cursor, Claude Code and others). This file is the source; `CLAUDE.md` only
imports it, because Claude Code reads that name. Edit this file, not that one.

Several sections are taken over from the RegelRecht repository
(https://github.com/MinBZK/regelrecht), where these tools started.

## Skills

Task-specific instructions live in `.claude/skills/<name>/SKILL.md`. When this
file names a skill (`code-reviewer`, `rust-errors`, ...), read that `SKILL.md`
before starting the task. Claude Code loads them on its own; other agents open
them as ordinary files.

- `code-reviewer` — reviewing a change (see "Code Reviews")
- `rust-errors`, `rust-testing`, `rust-api-design`, `rust-concurrency` — Rust
  code in a tool
- `storybook-component-hierarchy` — UI built from the NLDD design system

## Repository layout

Every tool is **standalone**: it lives in `tools/<name>/` with its own build
files, dependencies, lockfile, toolchain, tests, CI workflow
(`.github/workflows/<name>.yml`) and release tags (`<name>/v<version>`).

- **Never couple two tools.** No shared lockfile, workspace or package at the
  root, no import from another tool's folder. Code two tools need is copied
  deliberately, with its origin named in a comment, or becomes a tool of its
  own.
- **A tool never depends on a project it describes**, and no project depends
  on it. A tool gets the code it works on from its arguments (a path, a
  manifest), not from its own location.
- **Every tool has a `README.md` and a `justfile`** with at least `test` and
  `build`. CI calls those recipes, so it does not need to know the tool's
  language.
- **Adding a tool**: its folder, its workflow (triggered only by its folder),
  its Dependabot entries, its row in the root `README.md`, and in
  `publiccode.yml` when it is worth naming there.

## Where facts live

This file holds instructions: rules, conventions, what to do or avoid. Facts
about a tool (what it does, how it works, what it needs) live in its README,
and this file links to it instead of repeating it. When you change how a tool
works, update its README in the same change.

## Commits

**NEVER use `--no-verify` when committing.** Fix the underlying problem instead
of bypassing hooks. Install them with `pre-commit install`.

**No tool branding in commits.** Do not add "Generated with <tool>" or
"Co-Authored-By: <AI agent>" lines to commit messages.

### Commit & PR title conventions

The format is **Conventional Commits**: `type(scope): subject`, where `(scope)`
is optional and, when given, names the tool (`code-guide`) or `repo`, `ci`,
`deps`, `docs`.

- **Allowed types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`,
  `test`, `chore`, `build`, `ci`.
- **The subject starts with a lowercase letter**.
- Subjects and PR descriptions are written in **Dutch**; code identifiers stay
  English.

## Public repository

**Never use real secret or private information** in code, tests, fixtures,
comments or commit messages. This repository is public: anything in it is
published. No names of private repositories, internal hostnames, credentials,
tokens, personal data, or references to a private working environment. Use
clearly fictional placeholders (`acme-engine`, `example-org/example-repo`).

**Licences.** Everything is EUPL-1.2 unless `REUSE.toml` says otherwise, and
`reuse lint` must pass.

- A new file type gets a line in `REUSE.toml`. Images, fonts and logos get
  their own rights holder and licence, looked up, never the code licence by
  default.
- A new dependency must be under a licence the tool allows (`deny.toml` for
  Rust; for npm, the list in the tool's README).
- **Do not commit or publish built output that bundles third-party assets**
  without checking it may be redistributed. The NLDD design system bundles the
  Rijkshuisstijl fonts, which carry a copyright reservation of the State
  (`REUSE.toml`); `dist/` is ignored for that reason.

## Git Worktrees

When using git worktrees, create them **inside the project folder** (e.g.,
`.worktrees/`).

```bash
git worktree add .worktrees/feature-branch feature-branch
```

## When a source of truth is wrong

Fix the source, not a layer on top of it. A tool that reads another program's
output (an index, metadata) handles what that program really writes; it does
not patch around a misunderstanding in a fixture.

## Frontend / UI Components

**All user interface MUST be built with components from the NLDD design
system** (`@nldd/design-system`, https://github.com/MinBZK/storybook). Do not
hand-roll custom UI elements when a design-system component exists. For the
required component hierarchy, nesting rules, and layout patterns, use the
`storybook-component-hierarchy` skill.

The element prefix is `nldd-`, with two l's. Check an attribute against the
package's own `.d.ts` before using it: a web component with an attribute it
does not know renders nothing and reports nothing, so a guessed attribute fails
silently and only in the browser.

### Icon names

When an icon has an alias, prefer the **alias name** over the canonical name,
e.g. `icon="info"` not `info-circle`. The alias list lives in the design
system at `src/components/content/icon/icon-aliases.js`.

### When you can't build what you want with these components

Do **not** silently improvise. Follow these steps in order:

1. **Reconsider the design choice.** Try to conform to choices already made
   elsewhere in the tool before introducing anything new.
2. **If you still can't proceed: stop and ask.** Say explicitly that you need
   to take a shortcut, describe what's missing, and ask for permission.

### Reporting additional CSS

If you needed **any additional CSS styling** on top of the design-system
components (overrides, custom spacing, layout hacks, etc.), you **must report
this explicitly**: list exactly what custom CSS you added and why, and keep the
tool's README list of custom CSS up to date. Custom styling is a signal that may
need a design-system change, so it must never be hidden.

## Code Reviews

After completing significant code changes, review them with the
`code-reviewer` skill (`.claude/skills/code-reviewer/SKILL.md`) before
committing. Run that review in a fresh subagent when your tool has one (in
Claude Code: the Agent tool with `subagent_type: "general-purpose"`), so the
review does not inherit the reasoning that produced the change.
