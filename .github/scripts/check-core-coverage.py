"""Per-module coverage gate for setmeup core logic.

The governance spec (quality-gates) requires coverage FLOORS on core logical
modules (manifest, secrets, identity orchestration) and explicitly forbids a
blanket whole-crate percentage. This script reads cargo-llvm-cov's JSON report
and fails if any core module file falls below its configured floor.

Floors are per-module and set with judgment (governance "focused floor +
judgment"): modules whose logic is testable away from OS specifics get a high
floor; modules that are dominated by OS/binary/network integration surface
(identity: ssh-keygen, gpg, provider HTTP, WSL filesystem paths) get a lower
floor with the rationale documented here. DO NOT raise an integration-heavy
module's floor without also adding meaningful unit tests; DO NOT lower a
pure-logic floor without reason.

Usage:
    check-core-coverage.py <coverage-report.json>

Exit 0 = all core modules at/above their floor (or no Cargo project -> not enforced).
Exit 1 = a core module is below its floor, OR the report has files but none
         match the configured core modules (fail-closed: the gate must always
         be exercised once a Cargo project exists).
"""
import json
import sys

# Module name (file stem) -> floor percent. Rationale:
#   manifest: pure parsing/resolution logic -> high floor.
#   secrets:  vault crypto + backend logic, tested away from OS keyring -> high floor.
#   identity: restore-before-generate + registration + WSL backups; dominated by
#             ssh-keygen/gpg/HTTP/filesystem integration surface -> judgment floor.
FLOORS = {
    "manifest": 80,
    "secrets": 80,
    # identity: pure logic (restore-before-generate ordering, marker split,
    # dispatch, backup-root detection, chmod) is unit-tested and covered;
    # the bulk of the module is OS/binary/network integration (ssh-keygen, gpg,
    # provider HTTP, /mnt/c filesystem) that needs a live distro, deferred to
    # follow-up #23. Floor reflects the testable-away-from-OS logic only.
    "identity": 25,
}


def local_modules(files):
    """Yield (module, filename, percent) for core-logic source files."""
    for entry in files:
        name = entry.get("filename", "")
        summary = entry.get("summary", {})
        percent = summary.get("lines", {}).get("percent", 100.0)
        stem = name.rsplit("/", 1)[-1]
        for module in FLOORS:
            if stem == f"{module}.rs" or f"/{module}/" in name:
                yield (module, name, percent)


def main(argv):
    report_path = argv[1] if len(argv) > 1 else "coverage.json"

    with open(report_path) as fh:
        report = json.load(fh)

    # cargo-llvm-cov --json emits data[].files[]
    files = report.get("data", [{}])[0].get("files", [])
    matches = list(local_modules(files))
    if not matches:
        if not files:
            # No Cargo project yet (scaffold phase): gate not enforced.
            print("No core module files found in coverage report; nothing to gate.")
            return 0
        # Fail-closed: a Cargo project exists and produced files, but none matched.
        print(
            "ERROR: coverage report has files but none match configured core modules "
            f"({'/'.join(FLOORS)}). Failing closed rather than letting the gate bypass silently.",
            file=sys.stderr,
        )
        return 1

    failed = [(m, f, p) for (m, f, p) in matches if p < FLOORS[m]]
    for m, f, p in sorted(matches, key=lambda t: t[0]):
        mark = "FAIL" if p < FLOORS[m] else " ok "
        print(f"  [{mark}] {m:<10} {f:<40} {p:6.1f}% (floor {FLOORS[m]:.0f}%)")

    if failed:
        for m, f, p in failed:
            print(f"ERROR: core module {m} ({f}) below floor "
                  f"({p:.1f}% < {FLOORS[m]:.0f}%)", file=sys.stderr)
        return 1
    print("All core modules at or above their floors.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))