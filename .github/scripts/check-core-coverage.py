"""Per-module coverage gate for setmeup core logic.

The governance spec (quality-gates) requires a coverage FLOOR on core logical
modules (manifest, secrets, identity orchestration) and explicitly forbids a
blanket whole-crate percentage. This script reads cargo-llvm-cov's JSON report
and fails if any core module file falls below the floor.

Usage:
    check-core-coverage.py <coverage-report.json> [--floor 80]

Exit 0 = all core modules at/above floor (or no Cargo project -> not enforced).
Exit 1 = a core module is below the floor, OR the report has files but none
         match the configured core modules (fail-closed: the gate must always
         be exercised once a Cargo project exists).
"""
import json
import sys

CORE_MODULES = (
    "manifest",
    "secrets",
    "identity",
)


def local_modules(files):
    """Yield (module, filename, percent) for core-logic source files."""
    for entry in files:
        name = entry.get("filename", "")
        summary = entry.get("summary", {})
        percent = summary.get("lines", {}).get("percent", 100.0)
        # Match src/<module>.rs (or src/<module>/... ), repo src only
        if any(f"/{m}." in name or name.startswith(f"src/{m}") for m in CORE_MODULES):
            yield (next(m for m in CORE_MODULES if f"/{m}." in name or name.startswith(f"src/{m}")), name, percent)


def main(argv):
    report_path = argv[1] if len(argv) > 1 else "coverage.json"
    floor = 80.0
    if "--floor" in argv:
        floor = float(argv[argv.index("--floor") + 1])

    with open(report_path) as fh:
        report = json.load(fh)

    # cargo-llvm-cov --json emits data[].files[]
    files = report.get("data", [{}])[0].get("files", [])
    matches = list(local_modules(files))
    # Fail-closed: once a Cargo project exists and the report has files,
    # the gate MUST be exercised. If no core module matched, something is
    # wrong (module renamed/moved) and passing silently would be a bypass.
    if not matches and files:
        print(
            "ERROR: coverage report has files but none match configured core modules "
            f"{'/'.join(CORE_MODULES)} (e.g. src/<module>.rs or src/<module>/). "
            "Failing closed rather than letting the gate bypass silently.",
            file=sys.stderr,
        )
        return 1

    failed = [(m, f, p) for (m, f, p) in matches if p < floor]
    for m, f, p in sorted(matches):
        mark = "FAIL" if p < floor else " ok "
        print(f"  [{mark}] {m:<12} {f:<40} {p:6.1f}%")

    if failed:
        for m, f, p in failed:
            print(f"ERROR: core module {m} ({f}) below floor ({p:.1f}% < {floor:.0f}%)", file=sys.stderr)
        return 1
    print(f"All core modules at or above floor ({floor:.0f}%).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))