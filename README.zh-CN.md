<p align="center">
  <img src="docs/assets/cover.webp" width="220" alt="OneBrain：围着共享大脑工作的像素风 Agent">
</p>

<h1 align="center">OneBrain</h1>

<p align="center">
  <b>一群编程 Agent，共用一个大脑。</b><br>
  只有被 @ 的成员才会运行，而且它只读自己还没看过的内容。
</p>

<p align="center">
  <a href="README.md">English</a> · 简体中文 · <a href="README.de.md">Deutsch</a>
</p>

<p align="center">
  <img src="docs/assets/token-chart-zh.svg" width="820" alt="单轮 token 消耗：OneBrain 保持平稳，每次重读历史则持续上涨；整段对话节省 31% 到 81%">
</p>

## 不一样在哪里

|  | 常见的多 Agent 做法 | OneBrain |
| --- | --- | --- |
| Agent 跟上进度 | 重读历史 | 只读它没看过的新内容 |
| 新成员加入 | 拿到整段聊天记录 | 拿到一份简短的团队交接 |
| 谁会运行 | 所有人，或由调度 Agent 决定 | 只有你 @ 的那一位 |
| 记忆 | 各自存在每个 Agent 里 | 一个人也能读的共享大脑 |
| 运行中途你改了主意 | 接力照常进行 | 交接会等你 |
| 单轮成本 | 随对话越来越高 | 稳定在 1,000 token 左右 |

## 测试结果

三个成员（开发、审核、测试）在同一个对话里，每轮 @ 其中一位。两列提供给 Agent 的信息完全相同。

| 对话长度 | 每次重读历史 | OneBrain | 节省 |
| ---: | ---: | ---: | ---: |
| 10 轮 | 18,752 | 12,859 | **31%** |
| 30 轮 | 85,208 | 37,135 | **56%** |
| 60 轮 | 258,464 | 72,928 | **72%** |
| 100 轮 | 633,264 | 123,388 | **81%** |

中途加入的成员，比如第 100 轮才叫进来的第二位审核，起步只需要 1,611 个 token，而不是 11,364 个（少 **86%**）。

所有数字都来自同一条命令：用真实的 prompt 构建代码重放一段固定的对话。

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

测试方法和完整数据：[docs/benchmarks/context-tokens.md](docs/benchmarks/context-tokens.md)

## 怎么省下来的

不同厂商的 Agent 没法共用缓存，所以 OneBrain 不在成员之间传聊天记录。每个 Agent 保留自己的对话，成员之间只传小而稳定的内容。

**被 @ 才运行。** 没人 @ 的成员不运行，不花一个 token。

**每个成员一个已读位置。** 续接对话的成员只收到它上次之后的新消息；角色设定和团队规则没变，就不重发。只有运行成功，已读位置才会前进，失败的运行不会让消息被跳过。

**团队交接。** 新加入的成员会拿到你最近的请求、其他成员最近一次的结论和改动的文件，以及已经记录下来的决定。这些都来自会话里已有的数据，不需要额外调用模型去写总结。

**交接会等你。** Agent 干活期间你发了新消息，它发给其他成员的交接会显示出来，但不会叫醒对方。接力停在你的新消息这里，不会按旧的要求继续跑。

**每条回复都显示缓存命中率。** 每条回复下面写着新输入、缓存命中和输出的 token 数，以及命中缓存的比例。各家统计缓存的方式不同，OneBrain 会先统一口径再显示。

## 共享大脑

大多数 Agent 工具把学到的东西留在每个 Agent 自己那里：一段很长的聊天记录，或者只有它自己读的记忆文件。OneBrain 给整个团队一份共同的记忆，而且是写给人看的，不只是给 Agent 看。

设计如下：

- 决定、规则、经验和交接都是普通的 Markdown 文件。任何 Agent 工具都能打开，你能直接读，git 能看到每一次改动。
- 每一条都记着来源：出自哪条消息，或者描述的是哪个文件的哪几行。那些文件一改，这一条就会被标记为过期。
- Agent 只能提议，由你来批准。没有经过人看过的内容，不会变成团队知识。

已上线：共享记录、团队交接、每个成员的已读位置。进行中：App 里的大脑页面，可以浏览、批准和编辑每一条。

先有方法，后有 App。在 OneBrain 出现之前，同一台电脑上的几个编程 Agent 就已经靠共享的 Markdown 文件和一套成文的规则保持同步。OneBrain 把这个习惯做成了产品。

## 其他功能

- 暖白色的群聊界面，每种运行时配一个像素风头像，每位成员都有一张 GitHub 风格的每日运行热力图。
- 后台随 App 一起退出，崩溃或强制退出也一样，并且会停掉它启动的 Agent 进程。
- 每个对话可以用独立的 Git worktree，Agent 不会改到你正在用的工作目录。

支持 Claude Code、Codex、Gemini CLI、Qwen Code、OpenCode、Amp 等。

## 从源码构建

Rust（Axum、SQLx、SQLite）、React 与 TypeScript、Tauri。

```sh
pnpm install
pnpm run check
cargo test -p db -p services
```

## 致谢

Apache License 2.0。基于 [OpenTeams](https://github.com/openteams-lab/openteams)（同为 Apache-2.0）开发，详见 [NOTICE](NOTICE)。
