# Setup Integrations

Retrieval hooks: concepts: `trk setup claude`, `trk setup codex`,
`.claude/settings.local.json`, `.codex/config.toml`, `.codex/hooks.json`,
`.codex/rules/default.rules`, SessionStart, PreCompact, hook permissions.
Key files: `src/cmd/setup_claude.rs`, `src/cmd/setup_codex.rs`,
`src/main.rs`, `SPEC.md`. Useful when adding a new coding-agent target or
debugging why `trk prime` is not injected at session start.

Trapperkeeper integrates through agent hooks rather than static project
instructions. Setup commands are idempotent and preserve unrelated
existing config.

## Claude Code

`trk setup claude` edits `.claude/settings.local.json`.

It ensures:

- `hooks.SessionStart` contains a command hook for `trk prime`.
- `hooks.PreCompact` contains a command hook for `trk prime`.
- `permissions.allow` contains `Bash(trk:*)`.

The JSON merge is structural: existing hooks remain in place, and the
command is only appended when no existing hook already runs `trk prime`.
If existing `permissions.allow` is present but not an array, setup fails
instead of rewriting unknown config.

## Codex

`trk setup codex` edits three files under `.codex/`.

- `.codex/config.toml` — ensures `[features]` contains `hooks = true` and
  removes legacy `codex_hooks = ...` assignments from that table.
- `.codex/hooks.json` — ensures a `SessionStart` hook with matcher
  `startup|resume|clear`, command `trk prime`, and status message
  `Loading Trapperkeeper wiki context`.
- `.codex/rules/default.rules` — appends
  `prefix_rule(pattern=["trk"], decision="allow")` once.

Codex currently has no PreCompact equivalent in this implementation.
`src/cmd/setup_codex.rs` intentionally uses small line-oriented helpers
for the config file rather than introducing a TOML dependency.

## Prime Contract

Both integrations run the same `trk prime` command. `prime` is
mode-aware, so setup does not need to know whether the wiki is orphan-mode
or in-tree. In an uninitialized repo, `trk prime` prints nothing and exits
successfully, which keeps hooks harmless before `trk init`.
