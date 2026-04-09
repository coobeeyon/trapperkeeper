# Index

## C
- **clap** — CLI parsing framework. Cargo.toml, src/main.rs
- **commit_tree** — create git commit. src/git.rs:113, src/git.rs:118
- **CLAUDE.md** — not used; hooks replace it. pages/decisions.md

## G
- **git plumbing** — low-level git operations. src/git.rs, pages/git-plumbing.md

## H
- **hash_blob** — write blob to git object store. src/git.rs:26
- **hooks** — SessionStart/PreCompact integration. src/cmd/setup_claude.rs, pages/decisions.md

## I
- **ingest** — wiki operation: add source, update pages. pages/llm-wiki-pattern.md
- **init** — create orphan branch. src/cmd/init.rs, pages/architecture.md
- **insert_into_tree** — recursive tree manipulation. src/git.rs:135

## K
- **Karpathy LLM Wiki** — compile-don't-retrieve pattern. pages/llm-wiki-pattern.md, sources/karpathy-llm-wiki.md

## L
- **lint** — wiki operation: health check for contradictions/orphans. pages/llm-wiki-pattern.md
- **litebrite** — reference implementation. ../litebrite, SPEC.md, pages/decisions.md
- **LLM Wiki pattern** — three-layer architecture (raw/wiki/schema). pages/llm-wiki-pattern.md
- **ls_tree** — read tree entries. src/git.rs:91

## M
- **mktree** — build git tree. src/git.rs:55

## O
- **orphan branch** — wiki storage mechanism. pages/decisions.md, pages/architecture.md

## P
- **prime** — context injection command. src/cmd/prime.rs, pages/architecture.md

## Q
- **query** — wiki operation: ask questions against compiled pages. pages/llm-wiki-pattern.md

## S
- **schema drift** — LLM diverges from wiki conventions over time. pages/llm-wiki-pattern.md
- **setup_claude** — hook wiring command. src/cmd/setup_claude.rs, pages/architecture.md
- **settings.local.json** — Claude Code config. .claude/settings.local.json, src/cmd/setup_claude.rs

## T
- **trapperkeeper (branch)** — orphan branch name. src/git.rs:3

## W
- **write** — write file to orphan branch. src/cmd/write.rs, pages/architecture.md
- **write_file_to_tree** — high-level tree write. src/git.rs:129
