## ADDED Requirements

### Requirement: CODEOWNERS scaffold
The repository SHALL contain a `CODEOWNERS` file that assigns ownership on trust-boundary and governance paths, signaling governance awareness even in a single-maintainer repo.

#### Scenario: Change to governance path
- **WHEN** a PR modifies `.gitleaks/`, `.github/`, `AGENTS.md`, `docs/workflow.md`, or `openspec/`
- **THEN** the owning individual is surfaced by GitHub as the requested reviewer for that change
- **THEN** the PR still merges normally (the owner's review is a signal, not a mechanically required one — `require_code_owner_reviews` is off because a personal-repo code owner is the author; the mechanically enforced review gate is the `agent-review` check, see branch-protection/ci-pipeline)

#### Scenario: Change outside governance paths
- **WHEN** a PR modifies only product code outside governed paths
- **THEN** no CODEOWNERS-imposed required reviewer is added