<p align="center">
  <img src="docs/assets/cover.webp" width="200" alt="OneBrain: pixel-art agents around a shared brain">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>Your coding agents, in one group chat.</b><br>
  Claude Code, Codex and Gemini work together on your Mac.<br>
  Each one reads only what is new to it, so long conversations cost up to 81% fewer tokens.
</p>

<p align="center">
  <b>81% fewer tokens</b> over 100 turns · <b>~1,000 tokens</b> per turn, however long the chat · <b>0 tokens</b> while idle
</p>

<p align="center">
  <a href="https://github.com/0xTaoZ/onebrain/releases/latest"><b>⬇ Download for macOS</b></a>
  &nbsp;·&nbsp; English · <a href="README.zh-CN.md">简体中文</a> · <a href="README.de.md">Deutsch</a>
</p>

<p align="center">
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-7c5cff?style=flat-square">
  <img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon-1d1d1f?style=flat-square&logo=apple">
  <img alt="Local, no account" src="https://img.shields.io/badge/runs-locally%2C%20no%20account-2b7d4a?style=flat-square">
</p>

## Install

1. Download `OneBrain_x.y.z_aarch64.dmg` from [Releases](https://github.com/0xTaoZ/onebrain/releases/latest). It runs on Macs with Apple Silicon (M1 or later).
2. Open the file and drag **OneBrain** into **Applications**.
3. The first time you open it, macOS blocks apps that do not come from the App Store or a paid Apple developer account. Open **System Settings → Privacy & Security**, scroll down and click **Open Anyway**. If macOS says the app is damaged instead, run this once in Terminal:
   ```sh
   xattr -dr com.apple.quarantine /Applications/OneBrain.app
   ```
4. Have at least one coding agent installed and signed in, for example Claude Code, Codex or Gemini CLI. OneBrain uses your own subscriptions and never sees your keys. Create a group, add your agents, and mention one with `@`.

## Why I built it

I use several coding agents at once. Most of what they cost went into reading the same chat history again and again, and getting them to hand work to each other meant copying text between terminals. OneBrain puts them in one chat and gives each one only what it needs.

<p align="center">
  <img src="docs/assets/token-chart-en.svg" width="820" alt="Tokens for one turn stay flat with OneBrain while rereading history grows; savings of 31 to 81 percent">
</p>

## How it compares

|  | Session managers<br><sub>side-by-side terminals</sub> | Cloud group chats | OneBrain |
| --- | :---: | :---: | :---: |
| Works with agents from several vendors | ✓ | ✓ | ✓ |
| Agents hand work to each other in one chat | – | ✓ | ✓ |
| Runs locally, no account | ✓ | – | ✓ |
| A late joiner gets a short handoff instead of the history | – | – | ✓ |
| Measured token savings you can reproduce | – | – | ✓ |

## The numbers

Three agents (coder, reviewer, tester), one mentioned per turn. Both columns give the agents the same information.

| Conversation | Rereading history | OneBrain | Saved |
| ---: | ---: | ---: | ---: |
| 10 turns | 18,752 | 12,859 | **31%** |
| 30 turns | 85,208 | 37,135 | **56%** |
| 60 turns | 258,464 | 72,928 | **72%** |
| 100 turns | 633,264 | 123,388 | **81%** |

A reviewer called in after 100 turns starts with **1,611** tokens instead of **11,364**.

Every number comes from one command that replays a fixed conversation through the real prompt builder, so you can check it yourself:

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

Method: [docs/benchmarks/context-tokens.md](docs/benchmarks/context-tokens.md)

## How it works

Agents from different vendors cannot share a provider cache, so OneBrain does not pass transcripts around. Each agent keeps its own conversation, and only small, stable pieces move between agents.

- **Mention to run.** An agent runs only when someone mentions it.
- **A read cursor per agent.** A resumed agent gets the messages since its last run, not the whole history. Its role and the team rules are not resent while they stay the same.
- **Team handoffs.** A newcomer gets your latest requests, each other agent's latest conclusion and changed files, and the decisions recorded so far. It is built from data the chat already stores, so no model call is spent on a summary.
- **Handoffs wait for you.** If you post while an agent is working, its handoffs are shown but do not wake the next agent, so nobody keeps working on instructions you just changed.
- **Cache hit rate on every reply.** Fresh input, cached input and output, normalised across providers.

## The shared brain

Most agent tools keep what they learn inside each agent. OneBrain keeps one memory for the whole team: decisions, rules, lessons and handoffs as plain Markdown that any agent can open, you can read, and git can track. Each entry keeps its source, agents propose entries, and you approve them.

Live today: shared records, team handoffs and per-agent read cursors. Next: a brain page in the app to browse, approve and edit entries.

## Also in the app

- Warm white chat with pixel-art avatars for each agent runtime.
- A GitHub-style heatmap of each agent's daily runs.
- The backend closes with the app, even after a crash, and stops the agent processes it started.
- Optional Git worktree per conversation, so agents never write over your working copy.

## Build from source

Rust (Axum, SQLx, SQLite), React and TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

---

<sub>Apache License 2.0. Built on OpenTeams (Apache-2.0); see NOTICE.</sub>
