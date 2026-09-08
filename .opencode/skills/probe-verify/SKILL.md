---
name: probe-verify
description: Inspect actual machine and repository state before asserting anything about it. Use when an agent needs to claim a fact about the environment (tool presence, config state, file layout, build/test/provisioning outcome) or is about to act on an assumption about what exists or what happened.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Probe, don't assume

The single most expensive agent failure mode is **asserting state that was never verified**: assuming a tool is installed, claiming a build passed, guessing a file layout, trusting stale context. This skill exists to make verification the default cheap move.

## When to use

- Before claiming "X is installed / configured / running"
- Before proceeding on "the last session set up Y"
- Before reporting a task outcome (build, test, provisioning step)
- When the repo layout or environment is unfamiliar

## The probe reflex

1. **Inspect, don't recall.** Use the file tools / shell to check what actually exists. `ls`, `glob`/`grep`, `read` a config or a file header, check `which <tool>`, run the version flag.
2. **Read real output.** If you claim a command succeeded, you must have its output. Re-run it or read the log; do not reconstruct from memory.
3. **Check the target, check the boundary.** Verify not just that a file exists but that the *content* is as expected (a config file can be present and wrong).

## Ground rules

- If you haven't inspected it, you don't know it. Say "I haven't verified that" rather than asserting.
- When in doubt, probe again — state can change between two steps of a long workflow.
- Favor the cheapest sufficient probe: a single `ls`/`glob` over a heavyweight scan, a `read` over a re-run.

## After probing

- Record what you verified (short, concrete) instead of a generic "should be fine."
- If a probe contradicts what you expected, stop and reconcile — investigate before pushing forward.

## Maintenance

This skill must pay rent: if it ever stops mapping to real agent work, it is removed. Per `AGENTS.md`, unused skills are pruned — keep this doc current and genuinely used.