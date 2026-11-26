<p align="center">
  <img src="frontend/public/logos/onebrain-logo.png" width="200" alt="OneBrain: pixel-art agents around a shared brain">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>Your coding agents, in one group chat, built for developers.</b><br>
  Claude Code, Codex and Gemini talk to each other on your Mac, plan work together and review each other's code.<br>
  Each one reads only what is new to it, so long conversations cost up to 81% fewer tokens.
</p>

<p align="center">
  <b>81% fewer tokens</b> over 100 turns &nbsp;·&nbsp; <b>7× less</b> per turn by turn 100 &nbsp;·&nbsp; <b>0 tokens</b> while idle
</p>

<p align="center">
  <a href="#install"><b>⬇ Install for macOS</b></a>
  &nbsp;·&nbsp; English · <a href="README.zh-CN.md">简体中文</a> · <a href="README.de.md">Deutsch</a>
</p>

<p align="center">
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-7c5cff?style=flat-square">
  <img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon-1d1d1f?style=flat-square&logo=apple">
  <img alt="Local, no account" src="https://img.shields.io/badge/runs-locally%2C%20no%20account-2b7d4a?style=flat-square">
  <img alt="Release" src="https://img.shields.io/github/v/release/0xTaoZ/onebrain?style=flat-square&color=7c5cff">
</p>

## Three ways to use it

**Chat mode.** A group chat like any other. Mention an agent with `@` and only that agent answers. Ask Claude Code to build, Codex to review, Gemini to write the docs, all in one thread.

**Work mode.** Turn a discussion into a plan. The lead agent writes the steps, each step runs with the agent you assigned, and you approve the result before it counts as done.

**Built for developers.** Each conversation can get its own Git worktree, so agents never write over your working copy. Diffs, file changes, approvals and source control live next to the chat, and every reply shows its token use and cache hit rate.

<p align="center">
  <img src="docs/assets/token-chart-en.svg" width="820" alt="Tokens for one turn stay flat with OneBrain while rereading history grows; 7 times less by turn 100">
</p>

## Install

| | One-line install (recommended) | Download the `.dmg` |
| --- | --- | --- |
| **How** | Paste into Terminal:<br>`curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh \| bash` | Download `OneBrain_x.y.z_aarch64.dmg` from [Releases](https://github.com/0xTaoZ/onebrain/releases/latest), open it, drag **OneBrain** into **Applications** |
| **First launch** | Opens right away. The script installs the app and opens it. | macOS blocks apps from developers without a paid Apple account. Open **System Settings → Privacy & Security** and click **Open Anyway**. If it says the app is damaged, run `xattr -dr com.apple.quarantine /Applications/OneBrain.app` |
| **Needs** | A Mac with Apple Silicon (M1 or later) | Same |

Then install and sign in to at least one coding agent you already use, such as Claude Code, Codex or Gemini CLI. OneBrain uses your own subscriptions and never sees your keys. Create a group, add your agents, and mention one with `@`.

## How it compares

|  | OneBrain | Session managers<br><sub>Claude Squad, Conductor</sub> | Cloud group chats<br><sub>xMatrix, Pachat</sub> | Single-vendor teams<br><sub>Claude Code agent teams, Antigravity /swarm</sub> |
| --- | :---: | :---: | :---: | :---: |
| Agents from different vendors talk in one chat | ✅ | – | ✅ | – |
| Runs on your Mac, no extra account | ✅ | ✅ | – | ✅ |
| A late joiner gets a short handoff, not the whole history | ✅ | – | – | partly |
| Shared team memory that people can read | ✅ | – | ✅ | – |
| Published, reproducible token measurements | ✅ | – | – | – |

## Why I built it

I use several coding agents at once. Most of what they cost went into reading the same chat history again and again, and getting them to hand work to each other meant copying text between terminals. OneBrain puts them in one chat and gives each one only what it needs.

## The numbers

Three agents (coder, reviewer, tester), one mentioned per turn. Both columns give the agents the same information.

| Conversation | Rereading history | OneBrain | Saved |
| ---: | ---: | ---: | ---: |
| 10 turns | 18,752 | 12,859 | **31%** |
| 30 turns | 85,208 | 37,135 | **56%** |
| 60 turns | 258,464 | 72,928 | **72%** |
| 100 turns | 633,264 | 123,388 | **81%** |

A reviewer called in after 100 turns starts with **1,611** tokens instead of **11,364**. Every number comes from one command that replays a fixed conversation through the real prompt builder:

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

Method: [docs/benchmarks/context-tokens.md](docs/benchmarks/context-tokens.md)

## How it saves tokens

Agents from different vendors cannot share a provider cache, so OneBrain does not pass transcripts around. Each agent keeps its own conversation, and only small, stable pieces move between agents.

- **Mention to run.** An agent runs only when someone mentions it.
- **A read cursor per agent.** A resumed agent gets the messages since its last run. Its role and the team rules are not resent while they stay the same.
- **Team handoffs.** A newcomer gets your latest requests, each other agent's latest conclusion and changed files, and the decisions recorded so far, built from stored data without an extra model call.
- **Handoffs wait for you.** If you post while an agent is working, its handoffs do not wake the next agent, so nobody keeps working on instructions you just changed.

## The shared brain

Most agent tools keep what they learn inside each agent. OneBrain keeps one memory for the whole team: decisions, rules, lessons and handoffs as plain Markdown that any agent can open, you can read, and git can track. Each entry keeps its source, agents propose entries, and you approve them.

Live today: shared records, team handoffs and per-agent read cursors. Next: a brain page in the app to browse, approve and edit entries.

## Build from source

Rust (Axum, SQLx, SQLite), React and TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

---

<sub>Apache License 2.0. Built on OpenTeams (Apache-2.0); see NOTICE.</sub>
