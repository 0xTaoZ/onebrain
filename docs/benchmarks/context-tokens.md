# Context token benchmark

How many input tokens a mentioned agent receives per run, before and after the member read cursor.

## Results

Three members (coder, reviewer, tester), one member mentioned per turn, one reply per run. Token counts use the cl100k tokenizer.

| Turns | Before: prompt + history read | After | Saving | Last turn, before → after |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 18,752 | 12,859 | 31% | 2,313 → 1,392 |
| 30 | 85,208 | 37,135 | 56% | 4,289 → 1,035 |
| 60 | 258,464 | 72,928 | 72% | 7,194 → 1,041 |
| 100 | 633,264 | 123,388 | 81% | 11,348 → 1,469 |

Before the change, an agent that needs earlier context reads the session history file, which grows with every message, so the cost per turn grows with the conversation. After the change, a resumed agent receives only the messages it has not seen, so the cost per turn stays roughly flat.

Against the prompt alone (an agent that never reads history and therefore misses context), the saving is 10–15%. That part comes from not resending an unchanged role and team protocol.

### A new member joins

A member that has no view of the session yet, such as a second reviewer, starts from a team handoff instead of reading the history: the latest requests from the user, each other member's latest conclusion and changed files, and the shared records. The handoff is built from data the session already stores, so it costs no extra model call.

| Joins after | Before: prompt + history read | After: team handoff | Saving |
| ---: | ---: | ---: | ---: |
| 10 turns | 2,324 | 1,587 | 32% |
| 30 turns | 4,308 | 1,595 | 63% |
| 60 turns | 7,215 | 1,556 | 78% |
| 100 turns | 11,364 | 1,611 | 86% |

## Method

- Fixed conversation generated from a seeded pseudo-random sequence, so every run produces the same numbers.
- "Before": every run sends the full prompt; the history file read is counted as all earlier messages in JSONL form.
- "After": a member's first run gets the team handoff instead of the history read. Later runs get a short unchanged-prefix note and only the new messages.
- Prompts are built by the production prompt builder, not a copy of it.

Reproduce:

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```
