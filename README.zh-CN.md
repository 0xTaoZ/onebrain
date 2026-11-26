<p align="center">
  <img src="frontend/public/logos/onebrain-logo.png" width="200" alt="OneBrain：围着共享大脑工作的像素风 Agent">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>把你的编程 Agent 拉进一个群聊，专为程序员打造。</b><br>
  Claude Code、Codex、Gemini 在你的 Mac 上互相对话、一起排计划、互相审代码。<br>
  每个成员只读自己没看过的内容，长对话最多能省 81% 的 token。
</p>

<p align="center">
  100 轮对话 <b>省 81% token</b> &nbsp;·&nbsp; 第 100 轮单轮消耗 <b>少 7 倍</b> &nbsp;·&nbsp; 没被 @ 的成员 <b>0 token</b>
</p>

<p align="center">
  <a href="#安装"><b>⬇ 安装 macOS 版</b></a>
  &nbsp;·&nbsp; <a href="README.md">English</a> · 简体中文 · <a href="README.de.md">Deutsch</a>
</p>

<p align="center">
  <img alt="License Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-7c5cff?style=flat-square">
  <img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon-1d1d1f?style=flat-square&logo=apple">
  <img alt="本地运行，无需账号" src="https://img.shields.io/badge/本地运行-无需账号-2b7d4a?style=flat-square">
  <img alt="Release" src="https://img.shields.io/github/v/release/0xTaoZ/onebrain?style=flat-square&color=7c5cff">
</p>

## 三种用法

**聊天模式。** 和普通群聊一样。用 `@` 叫谁，就只有谁回复。让 Claude Code 写功能、Codex 审代码、Gemini 写文档，都在同一个群里。

**工作模式。** 把讨论变成计划。主 Agent 写出步骤，每一步由你指定的成员执行，最后由你验收，才算完成。

**为程序员打造。** 每个对话可以有独立的 Git worktree，Agent 不会改到你正在用的目录。diff、改动的文件、审批和代码提交都在聊天旁边，每条回复都显示 token 消耗和缓存命中率。

<p align="center">
  <img src="docs/assets/token-chart-zh.svg" width="820" alt="单轮 token：OneBrain 保持平稳，每次重读历史则持续上涨；第 100 轮少 7 倍">
</p>

## 安装

| | 一行命令安装（推荐） | 下载 `.dmg` |
| --- | --- | --- |
| **怎么装** | 粘贴到终端运行：<br>`curl -fsSL https://raw.githubusercontent.com/0xTaoZ/onebrain/main/install.sh \| bash` | 在 [Releases](https://github.com/0xTaoZ/onebrain/releases/latest) 下载 `OneBrain_x.y.z_aarch64.dmg`，打开后把 **OneBrain** 拖进 **应用程序** |
| **第一次打开** | 直接就能打开。脚本装好后会自动启动。 | macOS 会拦下没有付费开发者账号的应用。打开 **系统设置 → 隐私与安全性**，点 **仍要打开**。如果提示"已损坏"，在终端运行 `xattr -dr com.apple.quarantine /Applications/OneBrain.app` |
| **系统要求** | Apple 芯片的 Mac（M1 及以后） | 同左 |

然后装好并登录至少一个你常用的编程 Agent，比如 Claude Code、Codex 或 Gemini CLI。OneBrain 用的是你自己的订阅，不会接触你的密钥。新建一个群，拉进成员，用 `@` 叫一位开始干活。

## 和其他工具比

|  | OneBrain | 会话管理器<br><sub>Claude Squad、Conductor</sub> | 云端群聊<br><sub>xMatrix、Pachat</sub> | 单一厂商团队<br><sub>Claude Code agent teams、Antigravity /swarm</sub> |
| --- | :---: | :---: | :---: | :---: |
| 不同厂商的 Agent 在一个群里对话 | ✅ | – | ✅ | – |
| 在你的 Mac 上运行，无需额外账号 | ✅ | ✅ | – | ✅ |
| 中途加入的成员只读简短交接，不读全部历史 | ✅ | – | – | 部分 |
| 人也能读的团队共享记忆 | ✅ | – | ✅ | – |
| 公开、可复现的 token 测试 | ✅ | – | – | – |

## 为什么做这个

我平时同时用好几个编程 Agent。花掉的 token 大部分都用在它们一遍遍重读聊天记录上；想让它们互相交接，只能在几个终端之间来回复制。OneBrain 把它们放进同一个群，并且只给每个成员它真正需要的内容。

## 数据

三个成员（开发、审核、测试），每轮 @ 其中一位。两列提供给 Agent 的信息完全相同。

| 对话长度 | 每次重读历史 | OneBrain | 节省 |
| ---: | ---: | ---: | ---: |
| 10 轮 | 18,752 | 12,859 | **31%** |
| 30 轮 | 85,208 | 37,135 | **56%** |
| 60 轮 | 258,464 | 72,928 | **72%** |
| 100 轮 | 633,264 | 123,388 | **81%** |

第 100 轮才叫进来的审核员，起步只用 **1,611** 个 token，而不是 **11,364** 个。所有数字都来自同一条命令，用真实的 prompt 构建代码重放一段固定对话：

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

测试方法：[docs/benchmarks/context-tokens.md](docs/benchmarks/context-tokens.md)

## 怎么省下来的

不同厂商的 Agent 没法共用缓存，所以 OneBrain 不在成员之间传聊天记录。每个 Agent 保留自己的对话，成员之间只传小而稳定的内容。

- **被 @ 才运行。** 没人叫的成员不运行。
- **每个成员一个已读位置。** 续接的成员只收到上次之后的新消息；角色和团队规则没变就不重发。
- **团队交接。** 新成员拿到你最近的请求、其他成员最近的结论和改动的文件，以及已记录的决定，全部来自已有数据，不额外调用模型。
- **交接会等你。** Agent 干活时你发了新消息，它的交接不会叫醒下一位，没人会按你刚改掉的要求继续干。

## 共享大脑

大多数 Agent 工具把学到的东西留在每个 Agent 自己那里。OneBrain 给整个团队一份共同的记忆：决定、规则、经验和交接都是普通的 Markdown，任何 Agent 都能打开，你能直接读，git 能记录每次改动。每一条都记着来源，Agent 提议，由你批准。

已上线：共享记录、团队交接、每个成员的已读位置。下一步：App 里的大脑页面，可以浏览、批准和编辑。

## 从源码构建

Rust（Axum、SQLx、SQLite）、React 与 TypeScript、Tauri。

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

---

<sub>Apache License 2.0。基于 OpenTeams（Apache-2.0）开发，详见 NOTICE。</sub>
