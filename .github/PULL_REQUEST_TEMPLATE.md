## OpenSpec change

**Change:** <!-- name/path of the OpenSpec change, e.g. dev-governance -->
**Resolves:** <!-- issue(s) this PR closes, e.g. Closes #1 -->

## Summary

<!-- What does this change do, and why? Two to three sentences. -->

## Done-gate

- [ ] `cargo fmt --check` clean
- [ ] `clippy -D warnings` clean
- [ ] `cargo test` green
- [ ] Change's spec scenarios covered by tests (list the scenarios/requirements addressed)

## Review checklist

- [ ] Aligns with the change's spec/design and the issue it resolves
- [ ] Security-sensitive paths inspected (secrets, git hygiene, identity, credentials — trust boundary per `AGENTS.md`)
- [ ] Risk claim is credible (what could go wrong, and how is it mitigated)

## Notes for reviewer

<!-- Anything the reviewer should know: trade-offs, incomplete areas, follow-ups. -->