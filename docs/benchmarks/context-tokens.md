# Context token benchmark

How many input tokens a mentioned agent receives per run, before and after the member read cursor.

## Results

Three members (coder, reviewer, tester), one member mentioned per turn, one reply per run. Token counts use the cl100k tokenizer.

| Turns | Prompt + history read | Incremental input | Saved vs. history reread | Saved vs. prompt only | Last turn, reread → incremental |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 20,369 | 13,349 | 34.5% | 16.8% | 2,478 → 1,396 |
| 30 | 90,014 | 37,594 | 58.2% | 21.8% | 4,447 → 1,032 |
| 60 | 268,112 | 73,399 | 72.6% | 23.6% | 7,362 → 1,048 |
| 100 | 649,322 | 123,829 | 80.9% | 22.8% | 11,499 → 1,459 |


For the 100-turn replay, the prompt-only baseline used 160,346 input tokens. The same replay used 649,322 when the baseline also reread history. These baselines represent different amounts of input. Counts exclude provider prices, cache discounts, and output tokens.

At turn 100, a new reviewer received 1,775 input tokens in a handoff, compared with 11,528 for the history reread baseline, a reduction of 84.6%. The handoff is built from stored data without an extra model call.

## Method

- Fixed conversation generated from a seeded pseudo-random sequence, so every run produces the same numbers.
- "Before": every run sends the full prompt; the history file read is counted as all earlier messages in JSONL form.
- "After": a member's first run gets the team handoff instead of the history read. Later runs get a short unchanged-prefix note and only the new messages.
- Prompts are built by the production prompt builder, not a copy of it.

Reproduce:

```sh
for turns in 10 30 60 100; do
  BENCH_TURNS=$turns cargo test -p services --lib context_token_benchmark -- --ignored --nocapture
done
```
