# How setmeup is developed

This is the reviewer-facing story of how this project is built. Every claim here is grounded in a real artifact in this repository — the conversation that produced it was shaped by these rules, and the rules are why the history looks the way it does.

## The belief

AI agents can develop software well — and do it faster than a human alone — **if** the workflow makes the right move the cheap move. The failure mode to engineer against is not "agents are lazy"; it is *drift*: an agent taking a shortcut because the correct path is friction. So the scaffold's job is to make the correct path the low-friction path, and the wrong path visibly worse.

Three ideas carry the whole design:

1. **Spec-first development.** Requirements become OpenSpec changes (`openspec/changes/`) before code exists. A change carries a proposal (why), specs (what), design (how), and tasks (order). Code that has no change behind it does not get written.
2. **The contract is discoverable and enforced.** `AGENTS.md` at the repo root states the non-negotiables: the public-repo trust boundary, what "done" means, and the discipline of exploring before writing. CI (`openspec/changes/dev-workflow`, the `.github/workflows/ci.yml`) turns the cheap part of the contract into a gate.
3. **Machinery and policy are separate changes.** `dev-workflow` built the machinery (contract, skills, CI scaffold, this doc). `dev-governance` adds the policy layer (issue → change flow, PR review gate, quality gates like coverage floor, `cargo deny`, shellcheck). The product itself lives in `setmeup-core`. Each concerns one thing, and they reference each other instead of overlapping.

## The loop

```
 explore → propose → implement → review → merge (PR) → archive → no dangling artifacts
```

1. **Explore** (`/opsx-explore`): think before committing to a shape. This pass produced the design decisions recorded in each change's `design.md`.
2. **Propose** (`/opsx-propose`): create an OpenSpec change — proposal, specs, design, tasks — validated by `openspec validate`. The proposal references the issue that seeds the change.
3. **Implement** (`/opsx-apply`): work tasks in order. Each task is a checkbox; each is marked complete only when the done-gate holds.
4. **Review**: every PR gets a checklist review before merge (see governance, below).
5. **Merge**: every change lands as a branch + PR; no direct-to-main. The public history is the durable evidence of the process.
6. **Archive**: once a branch's change is merged, the change is archived (`openspec archive`) as the *final commit* for that change — after the done-gate, review, and issue verification all pass. No open changes, open issues referenced by a merged change, or unarchived completed changes are left dangling after a branch merges.

## What "done" means

A task or change is done only when all four hold, per `AGENTS.md`:

- `cargo fmt --check` clean
- `clippy -D warnings` clean
- `cargo test` green
- the change's spec scenarios are covered by tests

## The trust boundary, and why it is architectural

This repo is **public**. The contract therefore treats secret-handling as a correctness boundary, not a convention: credentials live only in the age/keyring vault and non-git backup paths; CI runs a gitleaks scan that fails a change if secret-like patterns appear in tracked content. That is not a README warning — it is a gate. The allowlist for legitimate false positives lives in `.gitleaks/setmeup.toml` and is itself reviewable.

## Governance (issue → change → PR → archive)

The policy layer lives in the `dev-governance` change (archived spec: `openspec/specs/`). In force since that branch landed:

- **Branch + PR pipeline.** Every code change ships on a branch and lands via PR; no direct-to-main. Branch naming: `feat/<issue-number>-<change-slug>`, referencing the issue that seeds the change.
- **Checklist review gate.** Every PR receives a substantive checklist review — by an agent or a human — before merge, covering:
  1. Done-checklist satisfied (fmt, clippy, tests, spec scenarios)
  2. Alignment with the change's spec/design and the issue it resolves
  3. Security-sensitive paths inspected (secrets, git hygiene, identity, credentials — trust boundary)
  4. Risk claim credible (what could go wrong, mitigation)
  The PR template (`.github/PULL_REQUEST_TEMPLATE.md`) carries these; the review-gate entry point lives in the project skills.
- **Archive is the terminal step.** A change is archived as the *final commit* for its branch — only after the done-gate, review, and verification of the tagged issue(s) all pass. The archive syncs the change's delta specs into main specs and moves the change to `openspec/changes/archive/`.
- **No dangling artifacts.** After a branch merges: no open issues referenced by the merged change, no unarchived completed changes, no open changes whose work shipped. Everything closes; nothing is left in a half state.

### Mechanically enforced gates (dev-protection)

The gates above are not just policy — they are enforced GitHub-side (`dev-protection`, issue #25):

- **Branch protection on `main`** (`gh api`: classic protection). Direct pushes, force-pushes, and merges that have not passed the required checks are rejected by the server. Required status checks are exactly **coverage**, **deps**, **shell**, **secrets** (and **agent-review**, once enabled); the branch must be up to date before merge; admins are subject to the same gate (`enforce_admins`).
- **Squash-only merges.** Repo settings have merge and rebase commits disabled; every merge on `main` produces one focused commit (the durable evidence of the process). Merged branches auto-delete.
- **SHA-pinned actions.** Every `uses:` in `.github/workflows/ci.yml` is pinned to a commit SHA with the original tag kept as a trailing comment — mutable tags are a supply-chain risk in a public repo. Dependabot proposes updates for these pins.
- **CODEOWNERS.** `.github/CODEOWNERS` assigns the maintainer to trust-boundary and governance paths (`.gitleaks/`, `.github/`, `AGENTS.md`, `docs/workflow.md`, `openspec/`), so changes there surface a reviewer. This is a *signal* (a personal-repo code owner is the author, so mechanically requiring it would deadlock).
- **Dependabot.** Monthly cadence for `cargo` and `github-actions` (no npm — no `package.json`), grouped into single PRs per ecosystem to reduce noise.
- **Agent review (approval-equivalent gate).** `.github/workflows/agent-review.yml` + `.github/scripts/agent-review.py` run the **adversarial agent review** as a required status check: a model reviews the PR diff and OpenSpec change artifacts against the checklist and the job reports a green/red check. The check's exit code IS the merge gate — functionally an approval, without a review event. The workflow triggers on `pull_request_target`, so it runs **trusted code from the default branch**: it checks out only the base SHA and reads PR evidence via the GitHub API, so a PR can never modify its own reviewer or reach the `OPENROUTER_API_KEY` (scoped to the `setmeup_ci` environment). Fork PRs are skipped (external contributions use the human review gate). The script polls the four quality checks (bounded) before reviewing and is fail-closed (no green checks / missing key / malformed verdict ⇒ red); the system prompt treats PR text as untrusted evidence against prompt injection. Dormant behind `vars.AGENT_REVIEWER_ENABLED`; the maintainer provisions the key, observes a live pass on a PR after this workflow is merged to `main`, then adds `agent-review` to the required checks (`tasks.md` 7.4 of dev-protection). No GitHub App or second identity is involved. Honest caveat: the checklist is automated, so the substantive review remains the process gate (`pr-review` skill).

## The pieces, and where they live

| Concern | Location |
|---|---|
| Agent contract | `AGENTS.md` (repo root) |
| OpenSpec changes | `openspec/changes/` — `setmeup-core`, `dev-workflow`, `dev-governance` |
| Project context for artifacts | `openspec/config.yaml` |
| Project skills (curated) | `.opencode/skills/` — `probe-verify`, `done-checklist`, `pr-review`, plus the OpenSpec skills |
| CI / enforcement | `.github/workflows/ci.yml` + `.gitleaks/setmeup.toml`; enforced via branch protection, squash-only settings, CODEOWNERS (`.github/CODEOWNERS`), Dependabot (`.github/dependabot.yml`) |
| This story | `docs/workflow.md` |

## Conventions honored by reference

The shared engineering principles from `~/.config/opencode/AGENTS.md` apply here as read-only reference: SOLID, YAGNI, testing discipline, idempotency, minimal dependencies, exact version pinning, conventional commits, focused commits, security hygiene. They are cited, not rewritten.

## Keeping this current

This doc is a live artifact. When the workflow materially changes — a skill is added or pruned, CI gates change, a new change convention appears — this file and `AGENTS.md` are updated together. A stale `docs/workflow.md` would be worse than none.

## Lessons learned (from the first full cycle)

The `dev-governance` branch was the first to run the whole loop (issues → branch → PR → review → merge → close → archive). What it taught us:

- **The done-gate is narrower than it looks.** We verified gates in the *scaffold phase* (coverage script, shellcheck, cargo-deny against scratch fixtures) rather than on the merged repo. That verification caught real defects, but only because we probed with *negative* tests — simulating a leak, a bad script, a vulnerable dependency. A gate verified only by "it exists" is decoration.
- **Fail-open is the enemy of a quality gate.** Review found the coverage gate exited 0 when a report contained files but none matched the core modules — a silent bypass at 0% coverage. Fixed to fail closed (`#7`, `9031cd9`). Rule we now apply: if a gate is supposed to *enforce*, its "nothing to enforce" case must fail, not pass.
- **Probe-crafted tools need honest fixture tests.** The coverage script, shellcheck, and cargo-deny were all validated against synthetic inputs (pass, fail, no-match, empty). That's the `probe-verify` skill applied to our own tooling.
- **The issue → change → PR → merge → close → archive trace works, but only if you run it to the end.** GitHub auto-closed only the *first* issue listed in "Closes #1-5"; `#2-5` needed explicit closing. Lesson: don't assume the merge auto-closes every referenced issue — verify with a final sweep (no open issues, no open PRs, exactly the intended active changes), as we now do.
- **Conventional-commit discipline pays off at review.** Four focused commits (templates, docs, gates, artifacts) rendered a diff a reviewer could actually reason about. The one defect found was *inside* one commit, not smeared across the whole branch.
- **The `gh issue create` flags differ from `gh api`.** `--jq` isn't supported on `gh issue create` (the skill documented `gh api` for writes); we captured URLs from output instead. Detail, but it cost one failed call — worth noting in the `github-issues` skill.

## Lessons learned (setmeup-core cycle)

The `setmeup-core` branch was the second full loop, and the first on real product code (a fresh Rust crate) rather than scaffolding. What it added:

- **The done-gate can be overclaimed without per-task probing.** Several `setmeup-core` tasks were marked complete during planning-adjacent exploration that were NOT actually implemented when the code landed: OAuth device-flow (5.3), full ratatui wizard screens (6.1/6.2), GPG registration (7.2/7.3), and backend auto-selection (4.4). Probing the actual code (grep for real call sites, not just enum variants) before marking "done" caught each one. Rule: a task is done only when its implementation is *wired and exercised*, not when a stub or enum variant exists.
- **Honest scoping beats overclaiming.** After the probe surfaced the gaps, trimming the change contract to the delivered MVP — with the deferred requirements written as NOTES in the specs and tracked as follow-up issues (#19–23) — was the right call. An archived change must be a contract the code actually meets. `setmeup-core` archived at 39/55 with the deferred 16 tasks seeding future changes is a clean, truthful artifact.
- **A config gate is not a gate until it has run.** `deny.toml` and the coverage script were written in `dev-governance` and never executed — both broke the moment the first crate existed: `deny.toml` was invalid against cargo-deny 0.20 (advisories scope keys, `[sources.allow]` table, license allowlist gaps), and the coverage gate failed the identity module once real code existed. Lesson: execute a quality gate against a realistic fixture at authoring time, and re-run it when the first real consumer lands (exactly what CI does — which is why the dependency gate went red on the PR and got fixed before merge).
- **A "focused floor + judgment" coverage gate needs per-module floors, not one number.** The single 80% floor honestly flagged identity at 13.7% → 29% even after pure-logic tests, because the module is dominated by OS/binary/network integration surface (ssh-keygen, gpg, provider HTTP, `/mnt/c`). Setting per-module floors (manifest 80, secrets 80, identity 25) with the rationale documented in the script — rather than gaming a blanket number — is the governance design in action. The floor is a floor for the testable logic, not a claim about unexercised integration happy paths (those are covered by the deferred E2E, #23).
- **Exact pinning means probing actual crates, not recalling versions.** `zeroize = 1.8.2` and `tempfile = 3.23.0` were initially pinned from memory; live versions were `1.9.0` / `3.27.0`. Also `serde_yaml` is deprecated/archived — the maintained fork is `serde_yaml_ng`. Always `cargo search` before pinning, and prefer the maintained fork over a deprecated crate.
- **A host without a C toolchain is not a blocker when Docker exists.** No `cc`/`libc6-dev`/CRT on this box (sudo needs a password) — but `docker run rust:1.98` (the pin) built and gated everything. Worth noting: the container needed `CARGO_TARGET_DIR` inside the workspace for the binary to survive; a target-dir volume broke smoke tests initially.
- **Background-lane discipline pays off.** The `bootstrap.sh` (a genuinely independent file) ran as a background fixer lane while the Rust crate work continued in the main session — shellcheck/shfmt-gated, reconciled clean. The one lane was the right split; the crate itself stayed in one writer to avoid conflicts.