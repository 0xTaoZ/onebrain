# Context token benchmark

How many input tokens a mentioned agent receives per run, before and after the member read cursor.

## Results

Three members (coder, reviewer, tester), one member mentioned per turn, one reply per run. Token counts use the cl100k tokenizer.

| Turns | Before: prompt + history read | After: read cursor | Saving | Last turn, before → after |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 18,751 | 13,048 | 30% | 2,319 → 1,398 |
| 30 | 85,185 | 37,302 | 56% | 4,289 → 1,035 |
| 60 | 258,484 | 73,138 | 72% | 7,198 → 1,045 |
| 100 | 633,331 | 123,645 | 80% | 11,346 → 1,467 |

Before the change, an agent that needs earlier context reads the session history file, which grows with every message, so the cost per turn grows with the conversation. After the change, a resumed agent receives only the messages it has not seen, so the cost per turn stays roughly flat.

Against the prompt alone (an agent that never reads history and therefore misses context), the saving is 10–15%. That part comes from not resending an unchanged role and team protocol.

## Method

- Fixed conversation generated from a seeded pseudo-random sequence, so every run produces the same numbers.
- "Before": every run sends the full prompt; the history file read is counted as all earlier messages in JSONL form.
- "After": a member's first run is counted the same as before, including the history read. Later runs get a short unchanged-prefix note and only the new messages.
- Prompts are built by the production prompt builder, not a copy of it.

Reproduce:

```sh
BENCH_TURNS=30 cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
```

## Limits

- cl100k is not the exact tokenizer of every model; percentages are more reliable than absolute counts.
- The "before" upper line assumes the agent reads the whole history on each run. Real agents read it only when they decide they need it, so real savings fall between the two "before" lines.
- Provider-side prompt caching is not modelled here. A real-run benchmark with recorded input, cache-read and output tokens and latency is the next step.
- About 1,000 tokens per run are the fixed output format instructions, which are still resent every run.
