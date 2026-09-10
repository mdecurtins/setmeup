## 1. Create the skill file

- [x] 1.1 Add `.opencode/skills/ci-gate/SKILL.md` with skill metadata (name, description, `ci-gate-agent` trigger) and the "Before opening a PR" checklist

## 2. Cross-reference from openspec-propose skill

- [x] 2.1 In `.opencode/skills/openspec-propose/SKILL.md`, after the artifact-creation steps, add: "Before the PR is opened: After all `applyRequires` artifacts are done, load the `ci-gate` skill and run its pre-PR checklist"

## 3. Cross-reference from pr-review skill

- [x] 3.1 In `.opencode/skills/pr-review/SKILL.md`, in the "Rules" section, add: "Before declaring a review pass or opening the PR, load the `ci-gate` skill and run its pre-PR checklist"

## 4. Verify

- [x] 4.1 Load the skill and run its checklist against an existing proposal (e.g. the dev-hooks proposal that required 5 rounds) — confirm each check produces a relevant finding or explicit pass
- [x] 4.2 Run `openspec validate --all` and verify the change is in a valid state
- [x] 4.3 Confirm `cargo fmt --check`, `clippy -D warnings`, and `cargo test` are green (n/a — no Rust changes)
