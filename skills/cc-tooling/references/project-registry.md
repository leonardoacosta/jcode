# Project Registry

> Moved from global `CLAUDE.md` § 7 by the CLAUDE.md-split pattern (`rules/TOOLING.md` §
> CLAUDE.md-Split Pattern) — per-project reference data, relevant only when working in that
> specific project. `CLAUDE.md` keeps a one-line pointer.

T3 Turbo · `pnpm` · dev ports `3100–3199` · paths `~/dev/<code>`.

> **Source of truth:** per-project `.claude/project.toml` (capability `stack-composition` +
> `commands/apply/references/project-toml-schema.md`). This table is a hint; the manifest wins.

| Code | Notes |
| --- | --- |
| `otaku-odyssey` `tc` `tl` `modern-visa` `styles-silas` `la` `civalent` `cx` `homelab` `if` `terraform-modules` | Standard T3 fleet |
| `priceless-config` | Org governance/docs (no app, no devPort). See `~/dev/priceless/priceless-config/README.md`. |
| `tl` | Directory renamed 2026-07-19: lives at `~/dev/priceless/tavern-ledger` (GitHub repo `tavern-ledger`), not `~/dev/tl`. Project code stays `tl` — do not rename references to the short code. |
| `xx` | **Paradigm-divergent** — Bun + Effect 4.0 + oxlint/oxfmt. Gates: `bun fmt \| bun lint \| bun typecheck \| bun run test`. Dev: `node scripts/dev-runner.ts dev`. **Do NOT load `t3-code-patterns`.** |
| `brown` `ws` | Brown & Brown corporate fleet — own remotes/ADO/Fortify, not part of cc's own git flow; `ws`/satellites carry the same cross-repo caveat as global `CLAUDE.md` § 5 "Any git write with ambiguity" row. |
| `nexus` (registry code `nx`) `mesh` `nova` `mx` | Personal automation stack — `mesh` (Go, `cmd/mx-*` command mesh) feeds `mx`-gateway (triage aggregation, see rules/TOOLING.md § Ambient Surfacing), `nx` is the Swift menubar/iOS/watchOS deck, `nova` (legacy code `nv`) is the executor layer. Gate-not-mirror doctrine: `guidance-stack-architecture` memory. |
| `installfest` (registry code `if`) | Personal dotfiles/chezmoi repo — shell config, ssh-mesh, cross-platform Mac/Arch/Windows tooling. Owns the fleet's canonical project registry (`home/projects.toml`, see `openspec/specs/fleet-project-registry/spec.md`). |
| `harness` `atlas` | Personal utility repos, local/homelab-docker deploy tier only — no devPort block. |
