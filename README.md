# Trapperkeeper

Trapperkeeper is a small Rust CLI for maintaining an LLM-oriented wiki inside a
codebase. The wiki stores compiled project knowledge so future coding-agent
sessions can read the table of contents, index, and synthesized pages before
re-deriving the same context from source.

The installed binary is `trk`.

## Installation

### Prerequisites

- Git
- Rust and Cargo, installed with [rustup](https://rustup.rs/) or your package
  manager

### Install from GitHub

```sh
cargo install --git https://github.com/coobeeyon/trapperkeeper.git --bin trk
```

Confirm the binary is available:

```sh
trk --help
```

### Install from a local checkout

```sh
git clone https://github.com/coobeeyon/trapperkeeper.git
cd trapperkeeper
cargo install --path . --bin trk
```

For development, build and test with:

```sh
cargo build
cargo test
```

## Quick Start

Run these commands from the root of the repository that should get a wiki.

Initialize the default orphan-branch wiki:

```sh
trk init
```

This creates the `trapperkeeper` branch, checks it out as the gitignored
`.trapper_keeper/` worktree, writes `.trapperkeeper.json`, and configures
`log.md` with `merge=union`.

Alternatively, store the wiki as a normal committed directory:

```sh
trk init --in-tree docs/wiki
```

If you already have a partial in-tree wiki, adopt it without clobbering existing
files:

```sh
trk init --in-tree docs/wiki --adopt
```

## Agent Setup

Trapperkeeper can wire `trk prime` into supported coding-agent startup hooks.

For Claude Code:

```sh
trk setup claude
```

For Codex:

```sh
trk setup codex
```

Both setup commands are idempotent and merge with existing configuration.

## Usage

Print the wiki instructions and current retrieval context:

```sh
trk prime
```

In initialized repositories, agents should read the wiki before exploring source
files:

- `toc.md` for top-down navigation
- `index.md` for lookup by concept, command, filename, or error text
- `pages/` for synthesized architecture, decisions, and module knowledge
- `log.md` for append-only wiki activity

There is no `trk write` command. Update wiki files directly when useful
understanding crystallizes.

For the default orphan-branch mode:

```sh
git -C .trapper_keeper status
git -C .trapper_keeper add -A
git -C .trapper_keeper commit -m "Update trapperkeeper wiki"
```

For in-tree mode, commit the wiki files with the rest of the repository.

## Commands

```text
trk init [--in-tree <PATH>] [--adopt]
trk setup claude
trk setup codex
trk prime
```

## Storage Modes

Default orphan mode keeps wiki history separate from code history while still
traveling with the repository through Git. In-tree mode keeps the wiki visible
in normal code review and is useful when human collaborators should read or edit
the wiki without using a separate worktree.
