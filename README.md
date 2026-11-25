# OneBrain

A local desktop workspace where several coding agents share one group chat and one reviewed project memory.

Agents only run when they are mentioned. A member that continues an earlier conversation receives the messages it has not seen yet instead of the whole history, and its role and team rules are not resent while they stay the same.

## Current work

- Group chat in a warm white theme, with pixel avatars per agent runtime.
- Per-member read cursor: a resumed agent gets only new messages; the cursor advances only after a run succeeds.
- Desktop backend lifecycle: the backend shuts down cleanly when the app quits, crashes or is force-quit, and stops the agent processes it started.
- Contribution heatmap of each agent's daily runs over the last six months.

Planned: review results with attached test evidence, a shared project memory that records decisions with their sources, and per-member avatar choice.

## Stack

Rust (Axum, SQLx, SQLite) for the backend, React and TypeScript for the frontend, Tauri for the desktop shell. Supported agent runtimes include Claude Code, Codex, Gemini CLI, Qwen Code, OpenCode and Amp.

## Development

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

## License and credits

Apache License 2.0. OneBrain is built on [OpenTeams](https://github.com/openteams-lab/openteams), also Apache-2.0; see `NOTICE`.
