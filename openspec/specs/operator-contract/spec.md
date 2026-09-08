# operator-contract Specification

## Purpose
TBD - created by archiving change dev-workflow. Update Purpose after archive.
## Requirements
### Requirement: Root operator contract exists
The repository SHALL contain a root-level `AGENTS.md` that defines how agents work in this repository.

#### Scenario: Discovery
- **WHEN** an agent or human looks for development conventions at the repo root
- **THEN** a root `AGENTS.md` exists and is readable

### Requirement: Contract encodes the trust boundary
The contract SHALL state that secret material never enters git-tracked paths in this public repository, and that credentials live only in the vault/backup paths.

#### Scenario: Secret-handling rule
- **WHEN** an agent performs any credential or secret operation in this repo
- **THEN** the contract directs that secret material is written only to non-git vault or backup paths

#### Scenario: Public-repo awareness
- **WHEN** an agent reads the contract
- **THEN** it states that nothing secret may enter the repository because the repository is public

### Requirement: Contract encodes edit discipline
The contract SHALL direct that OpenSpec changes are the vehicle for development, and that code is not written before specs exist.

#### Scenario: New capability
- **WHEN** an agent wants to build a new feature or capability
- **THEN** the contract directs creating or updating an OpenSpec change first

### Requirement: Contract defines "done"
The contract SHALL define the completion gate for any work item as: formatting clean, `clippy -D warnings` clean, tests green, and spec scenarios covered.

#### Scenario: Work declared complete
- **WHEN** an agent reports a task or change as complete
- **THEN** the completion gate requires fmt clean, clippy clean, tests green, and spec scenarios covered

### Requirement: Explore-first default
The contract SHALL direct agents to understand context before writing, using available exploration affordances.

#### Scenario: Ambiguous request
- **WHEN** an agent receives a request that lacks context it needs to proceed correctly
- **THEN** the contract directs it to explore or ask before writing code

