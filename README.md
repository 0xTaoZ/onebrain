<p align="center">
  <img src="docs/assets/cover.webp" width="220" alt="OneBrain: pixel-art agents around a shared brain">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>A team of coding agents with one shared brain.</b><br>
  Only the agent you mention runs, and it reads only what it has not seen.
</p>

<p align="center">
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-7c5cff?style=flat-square">
  <img alt="macOS" src="https://img.shields.io/badge/platform-macOS-1d1d1f?style=flat-square&logo=apple">
  <img alt="Rust and Tauri" src="https://img.shields.io/badge/built%20with-Rust%20%C2%B7%20Tauri-b7410e?style=flat-square&logo=rust">
  <img alt="Tokens saved" src="https://img.shields.io/badge/tokens%20saved-31%E2%80%9381%25-2b7d4a?style=flat-square">
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a> · <a href="README.de.md">Deutsch</a>
</p>

<p align="center">
  <img src="docs/assets/token-chart-en.svg" width="820" alt="Tokens for one turn stay flat with OneBrain while rereading history grows; savings of 31 to 81 percent">
</p>

## What is different

|  | Common multi-agent setup | OneBrain |
| --- | --- | --- |
| An agent catches up | rereads the history | reads only what is new to it |
| A new agent joins | gets the transcript | gets a short team handoff |
| Who runs | everyone, or an orchestrator that decides | only the agent you mention |
| Memory | kept inside each agent | one shared brain that people can read |
| You change your mind mid-run | the chain keeps going | handoffs wait for you |
| Cost of one turn | grows with the conversation | stays around 1,000 tokens |

## Results

Three agents (coder, reviewer, tester) in one conversation, one agent mentioned per turn. Both columns give the agents the same information.

| Conversation length | Rereading history | OneBrain | Saved |
| ---: | ---: | ---: | ---: |
| 10 turns | 18,752 | 12,859 | **31%** |
| 30 turns | 85,208 | 37,135 | **56%** |
| 60 turns | 258,464 | 72,928 | **72%** |
| 100 turns | 633,264 | 123,388 | **81%** |

An agent that joins late, such as a second reviewer called in after 100 turns, starts with 1,611 tokens instead of 11,364 (**86%** less).

Every number comes from one command that replays a fixed conversation through the real prompt builder:

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

Method and all rows: [docs/benchmarks/context-tokens.md](docs/benchmarks/context-tokens.md)

## How it saves tokens

Agents from different vendors cannot share a provider cache. So OneBrain does not pass transcripts around. Each agent keeps its own conversation, and only small, stable pieces move between agents.

**Mention to run.** An agent runs only when someone mentions it. Idle agents cost nothing.

**A read cursor per agent.** A resumed agent receives the messages posted since its last run. Its role and the team rules are not resent while they stay the same. The cursor moves only after a run succeeds, so a failed run never hides messages.

**Team handoffs.** An agent new to a conversation gets your latest requests, each other agent's latest conclusion and changed files, and the decisions recorded so far. It is built from data the session already stores, so no model call is spent on writing a summary.

**Handoffs wait for you.** If you post while an agent is working, its handoffs to other agents are shown but do not wake them. The chain stops at your new message instead of running on old instructions.

**Cache hit rate on every reply.** Each reply shows fresh input, cached input and output, with the share served from the provider cache. Providers count cached tokens differently, and OneBrain normalises them before showing a rate.

## The shared brain

Most agent setups keep what they learn inside each agent: a long chat history, or memory files only that agent reads. OneBrain keeps one memory for the whole team, written for people as much as for agents.

The design:

- Decisions, rules, lessons and handoffs are plain Markdown. Any agent CLI can open them, you can read them, and git shows how they changed.
- Every entry keeps its source: the message it came from, or the file and lines it describes. When those files change, the entry is flagged as out of date.
- Agents propose entries; you approve them. Nothing becomes team knowledge without a person seeing it.

Live today: shared records, team handoffs and per-agent read cursors. In progress: a brain page in the app to browse, approve and edit entries.

The method came before the app. Several coding agents on one machine were already kept in step through shared Markdown files and a written set of rules. OneBrain turns that working habit into a product.

## Also in the app

- Group chat in a warm white theme, with pixel-art avatars for each agent runtime and a GitHub-style heatmap of each agent's daily runs.
- The backend shuts down with the app, even after a crash or force quit, and stops the agent processes it started.
- Optional Git worktree per conversation, so agents never write over your working copy.

Works with Claude Code, Codex, Gemini CLI, Qwen Code, OpenCode, Amp and more.

## Build from source

Rust (Axum, SQLx, SQLite), React and TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

## Credits

Apache License 2.0. Built on [OpenTeams](https://github.com/openteams-lab/openteams), also Apache-2.0; see [NOTICE](NOTICE).
