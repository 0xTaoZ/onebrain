<p align="center">
  <img src="frontend/public/logos/onebrain-logo.png" width="88" alt="OneBrain app icon">
</p>

<h1 align="center">OneBrain Rooms</h1>

<p align="center">
  <b>Claude builds. Codex reviews. One chat.</b><br>
  Open-source group chats for you and your agents on macOS. Chat, work on projects together, and keep team decisions in versioned Markdown.
</p>

<p align="center">
  <a href="https://github.com/0xTaoZ/onebrain/releases/latest"><b>Download for Apple Silicon</b></a>
  · <a href="#install">Install</a>
  · <a href="#compare-workflows">Compare</a>
  · <a href="#the-numbers">Token benchmark</a>
</p>

<p align="center">English · <a href="README.zh-CN.md">简体中文</a> · <a href="README.de.md">Deutsch</a></p>

<p align="center">
  <a href="docs/assets/demo/onebrain-demo.mp4">
    <img src="docs/assets/demo/onebrain-demo.gif" width="920" alt="OneBrain demo: Claude builds autosave, Codex catches a draft-loss bug, and Gemini records the fix in Markdown">
  </a><br>
  <sub>Example project rendered in OneBrain with sample data.</sub>
</p>

<p align="center">
  <a href="docs/assets/demo/onebrain-demo.mp4">Watch the MP4</a>
  · <a href="docs/assets/demo/group-chat.png">Group chat screenshot</a>
  · <a href="docs/assets/demo/team-brain.png">Shared brain screenshot</a>
</p>

## Why I built it

I wanted a place where I could ask a few agents to think through a question, come back the next day, and carry the decisions into the next task. On code projects I kept copying the request, the diff and the last answer from one terminal to another just to ask for a second opinion. Once the reviewer found something, I had to copy it back again.

In a chat room I can ask two agents for different perspectives. In a project room I can give them a working folder, hand a change to a reviewer, and check the diff before accepting it. When the team settles on a decision, it goes into Markdown that the next agent can read.

**Our goal is to build the most useful multi-agent workspace we can and make it use as few tokens as possible.** We measure progress on repeatable conversations and keep the benchmark in the repo.

## What makes OneBrain Rooms different

- **Cross-vendor review.** Mention Claude Code, Codex, or Gemini to assign the next step. In a project group, inspect diffs and approvals alongside the chat. Run checkpoints can undo file changes and flag later edits.
- **Context per agent.** A returning agent receives new messages. A new reviewer gets a short handoff assembled from stored data. **80.9% fewer input tokens** versus replaying the full history in a fixed 100-turn benchmark. [Reproduce the benchmark](docs/benchmarks/context-tokens.md).
- **Shared Markdown.** Team decisions live in files the coding CLIs can open directly, with Git history for authorship and restore.

## Install

The app appears as **OneBrain** in Applications.

| | One-line install (recommended) | Download the `.dmg` |
| --- | --- | --- |
| **How** | Paste into Terminal:<br>`curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh \| bash` | Download `OneBrain_x.y.z_aarch64.dmg` from [Releases](https://github.com/0xTaoZ/onebrain/releases/latest), open it, drag **OneBrain** into **Applications** |
| **First launch** | Opens right away. The script installs the app and opens it. | If macOS blocks the downloaded build, open **System Settings → Privacy & Security** and click **Open Anyway**. If it says the app is damaged, run `xattr -dr com.apple.quarantine /Applications/OneBrain.app` |
| **Needs** | A Mac with Apple Silicon (M1 or later) | Same |

Then install and sign in to at least one coding agent you already use, such as Claude Code, Codex or Gemini CLI. Agent access and billing follow the CLI or provider you configure. Create a group, add your agents, and mention one with `@`.

## First chat

1. On the welcome screen, pick a detected Agent and click **Add**.
2. Choose a chat group or a project group. A project group also needs a working folder.
3. Click a sample to send your first message, or open the chat and write your own.

## The numbers

<p align="center">
  <img src="docs/assets/token-chart-en.svg" width="840" alt="Cumulative input tokens at 10, 30, 60 and 100 turns. At 100 turns: 649,322 with full-history reread and 123,829 with OneBrain, a reduction of 80.9%; 22.8% versus the prompt-only baseline.">
</p>

Fixed three-agent conversation with one mention per turn. Counts are input tokens from the application’s prompt builder, measured with cl100k.

| Turns | Prompt + history reread | OneBrain | Saved vs. rereading | Saved vs. prompt only |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 20,369 | 13,349 | **34.5%** | 16.8% |
| 30 | 90,014 | 37,594 | **58.2%** | 21.8% |
| 60 | 268,112 | 73,399 | **72.6%** | 23.6% |
| 100 | 649,322 | 123,829 | **80.9%** | 22.8% |

A reviewer joining at 100 turns starts with 1,775 input tokens versus 11,528 when rereading history. These are benchmark measurements, not provider bills.

[Method](docs/benchmarks/context-tokens.md)

## Project controls

- **Work mode.** Turn a discussion into a plan, assign steps to agents, and approve the result.
- **Project worktrees.** Keep a group’s changes in a separate Git worktree and inspect the diff from the chat.
- **Schedules and activity.** Schedule group messages and review the activity log alongside approvals and file changes.

## Compare workflows

| Project | Where it runs | Claude + Codex | File undo | Git-backed team Markdown | Read cursor per agent | Reproducible token replay |
| --- | --- | :---: | :---: | :---: | :---: | :---: |
| **OneBrain Rooms** | Local app | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Pachat](https://pachat.ai/use-cases/agent-teams) | Hosted chat | ✅ | — | — | — | — |
| [Octo](https://www.mininglamp.com/octo/) | Self-host option | [◐](https://github.com/Mininglamp-OSS/octo-runtime) | — | — | — | — |
| [OpenAgents](https://openagents.org/faq) | Self-host option | ✅ | — | — | — | — |
| [AionUi](https://github.com/iOfficeAI/AionUi#team-mode--coordinated-multi-agent-collaboration) | Local app | ✅ | — | — | — | — |
| [PiBot](https://pibot.space/en/guides/bots-and-roles/#external-agents) | Local app | ✅ | [✅ macOS](https://github.com/w01fb0ss/PiBot#what-it-does) | — | — | — |
| [LobeHub](https://github.com/lobehub/lobehub) | Self-host option | ✅ | — | — | — | — |
| [Claude Code Agent Teams](https://code.claude.com/docs/en/agent-teams) | Local CLI | ✗ | — | — | — | — |

**✅** documented · **◐** documented at the runtime level · **✗** outside the documented product scope · **—** no comparable feature found in the linked docs.

“Claude + Codex” means both can join the same product workspace. “File undo” covers restoring files changed by an agent. The [token benchmark](docs/benchmarks/context-tokens.md) is our fixed conversation replay.

## How it works under the hood

| Mechanism | What happens in the app |
| --- | --- |
| [Incremental prompts](crates/services/src/services/chat_runner/prompting.rs) | The runner records a read cursor for each agent session. It checks the stable prefix and finds messages after the last entry the agent saw. A new member can start with a handoff assembled from stored data. |
| [Team memory](crates/services/src/services/brain.rs) | The shared brain is a folder of Markdown files. Git records changes and their authors, and the app can open older versions. |
| [Git checkpoints](crates/git/src/checkpoint.rs) | The app can roll back files touched by a run. Later edits to those files trigger a conflict check before undo writes anything. |
| [Scheduled chat tasks](crates/services/src/services/scheduler/mod.rs) | The scheduler claims a due job before posting through the normal group-chat path, where the same approval rules apply. |

## How we want to use AI

I'm building this by studying how people who make AI tools actually use them. One lesson I took from Lauren Tan’s [pstack guide](https://github.com/cursor/plugins/blob/main/pstack/docs/guide/README.md) is to say what would count as done before anyone starts. For a bug fix, give one agent the repro, ask another to check the diff and rerun the failing case, then write down what the team decided.

I want that to feel as easy as a group chat. Give each agent a job, show the reviewer what changed, and leave a note the next agent can pick up. Lauren’s [brainmaxxing](https://github.com/poteto/brainmaxxing) gives me a useful example of memory in editable files. [Noodle](https://github.com/poteto/noodle) does the same for repeatable workflows. If I just need a second opinion, I want to call one reviewer, give it the diff and the relevant messages, and see which check it ran and what happened.

## How it saves tokens

OneBrain tracks what each agent has already seen, so it can send new messages and a short handoff when another agent joins.

- **Choose who answers.** Use @ to name one agent, or @all to ask the group. A message without a mention goes to one selected agent.
- **A read cursor per agent.** A resumed agent gets the messages since its last run. Its role and the team rules are not resent while they stay the same.
- **Team handoffs.** A newcomer gets your latest requests, each other agent's latest conclusion and changed files, and the decisions recorded so far, built from stored data without an extra model call.
- **Handoffs wait for you.** If you post while an agent is working, its handoffs do not wake the next agent, so nobody keeps working on instructions you just changed.

## The shared brain

Most agent tools keep what they learn inside each agent. OneBrain keeps one memory for the whole team: decisions, rules, lessons and handoffs as plain Markdown that any agent can open, you can read, and git can track. Each entry keeps its source, agents propose entries, and you approve them.

The Brain page shows shared entries and handoffs.

## Build from source

Rust (Axum, SQLx, SQLite), React and TypeScript, Tauri.

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

## Follow the project

Star OneBrain Rooms to follow its development, or share what happened in your first group on [GitHub Issues](https://github.com/0xTaoZ/onebrain/issues).

---

<sub>Apache License 2.0. Built on OpenTeams (Apache-2.0); see NOTICE.</sub>
