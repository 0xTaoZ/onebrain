---
name: context-economy
description: Reduce repeated source reads and oversized tool output during multi-agent coding, research, and handoff. Use for long histories, repeated file inspection, shared Markdown knowledge, or token-cost concerns.
license: MIT
---

# Context economy

Keep the task correct and evidence retrievable. Use this skill on demand; do not paste this whole document into every prompt.

- Start with the task, constraints, current result, and next action. Keep handoffs short and attach source paths, message IDs, code versions, and test results.
- Search filenames and symbols before reading. Read the smallest relevant range; do not repack whole repositories or repeatedly dump the same file/page/log.
- Reuse a prior read only when its content is still in your active context and the source hash, path, range, and clipping settings match. A new agent or compacted session must retrieve the needed excerpt again. A shared hash alone does not give another model the contents.
- Keep stable instructions/tool definitions stable. Put changing task data later where the client allows. Provider prefix caching usually reduces input cost; cache creation and subscription quotas have provider-specific accounting.
- Retain raw logs and source messages; present short observations plus references. Never hide failures, change exit codes, or treat clipping as a model-generated summary.
- Read shared Markdown by topic and path. Treat summaries as derived, versioned evidence. Do not persist guesses or concurrent task state as shared facts.
- Delegate only separable work. Give each worker a bounded input and return contract. More agents can increase total tokens even when faster.
- AI-generated summaries require explicit opt-in and their own usage accounting. Do not start background inference, polling, wake/check-in jobs, or model downloads to save tokens.
- Do not promise a saving percentage. Measure payload bytes or reported token fields and label estimates, unknown values, and test fixtures accurately.

## Small reads without a persistent cache

Run the helper relative to this skill's directory:

```sh
python3 scripts/context_io.py read /absolute/source.py --start 40 --lines 50 --max-chars 4000
python3 scripts/context_io.py read /absolute/source.py --known-read-key PREVIOUS_READ_KEY --start 40 --lines 50
```

Only pass `--known-read-key` when the prior excerpt remains available in this agent's current context. Use `--force` to read again. No fingerprints or private source are stored automatically.

## Bounded command observations

```sh
python3 scripts/context_io.py run --log /absolute/task-output/check.log --max-chars 3000 -- python3 -m unittest
```

The complete combined output stays in the named local log. The helper reports its path/hash and the original exit status. For failures, inspect the full relevant error range before concluding. Avoid commands that expose credentials.
