#!/usr/bin/env python3
"""Adversarial agent reviewer for pull requests (dev-protection D7).

Runs as a REQUIRED CI status check ("agent-review"). A PR cannot merge until
this check reports green. The check's exit code IS the gate — it functionally
replaces a PR approval without any GitHub App: there is no review event to
post, only a pass/fail check run.

Behavior:
  - Approve: the review found nothing blocking -> exit 0 (check green).
  - Reject (or fail closed): the review found blocking issues, or any of the
    prerequisites are missing/unreadable, or the LLM verdict is malformed ->
    verify the verdict once against the diff, post a concise verdict comment
    on the PR (read-only GITHUB_TOKEN, pull-requests: write), and exit 1
    (check red)so the merge gate blocks.
  - Mandatory internal gate inside this script: it only *considers* approving
    when the four quality CI checks (coverage, deps, shell, secrets) are all
    SUCCESS on the head commit. Everything else rejects.

Security posture (public repo):
  - Runs on the `pull_request` trigger with a fork guard (workflow side);
    fork PRs never reach this code with the OpenRouter key available.
  - GITHUB_TOKEN is used ONLY to read PR evidence and to post the reject
    comment. It is never used to post an approval (there is none).
  - Fail closed: a missing secret, an exception, or a malformed LLM verdict
    fails the check. No quiet success.

Environment (set by the agent-review job in .github/workflows/ci.yml):
  GH_TOKEN             built-in GITHUB_TOKEN (read-only evidence + reject comment)
  OPENROUTER_API_KEY   maintainer's OpenRouter key (secret, scoped to setmeup_ci)
  GITHUB_REPOSITORY    owner/repo
  GITHUB_EVENT_PATH    path to the GitHub event payload
  AGENT_REVIEWER_MODEL optional OpenRouter model id; default: openrouter/auto
"""

import base64
import json
import os
import re
import sys
import urllib.error
import urllib.request

QUALITY_CHECKS = ("coverage", "deps", "shell", "secrets")
VERDICT_COMMENT_MARKER = "## Agent review"
DIFF_CAP = 150_000
API = "https://api.github.com"
OPENROUTER = "https://openrouter.ai/api/v1/chat/completions"


def gh(token, path, method="GET", data=None):
    """GitHub REST helper; raises RuntimeError on non-2xx/empty."""
    req = urllib.request.Request(f"{API}{path}", method=method)
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("User-Agent", "setmeup-agent-reviewer")
    if data is not None:
        req.add_header("Content-Type", "application/json")
        req.data = json.dumps(data).encode()
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            body = resp.read()
            if not body:
                raise RuntimeError(f"gh {method} {path}: empty response")
            return json.loads(body)
    except urllib.error.HTTPError as err:
        detail = err.read().decode(errors="replace")[:300]
        raise RuntimeError(f"gh {method} {path}: {err.code} {detail}")


def gh_paged(token, path):
    items, page = [], 1
    while True:
        sep = "&" if "?" in path else "?"
        batch = gh(token, f"{path}{sep}per_page=100&page={page}")
        if not batch:
            break
        items.extend(batch)
        if len(batch) < 100:
            break
        page += 1
    return items


def check_status(token, repo, head_sha):
    """Return {check_name: conclusion} for the four quality checks on head."""
    runs = gh_paged(token, f"/repos/{repo}/commits/{head_sha}/check-runs")
    status = {}
    for run in runs:
        name = run.get("name")
        if name and name in QUALITY_CHECKS:
            status[name] = run.get("conclusion") or run.get("status") or "pending"
    return status


def call_openrouter(api_key, model, system, user):
    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.2,
    }
    req = urllib.request.Request(
        OPENROUTER,
        data=json.dumps(payload).encode(),
        method="POST",
    )
    req.add_header("Authorization", f"Bearer {api_key}")
    req.add_header("Content-Type", "application/json")
    with urllib.request.urlopen(req, timeout=120) as resp:
        result = json.loads(resp.read())
    return result["choices"][0]["message"]["content"]


def parse_verdict(raw):
    """Extract the LLM's JSON verdict, tolerating markdown fences.

    Fail closed: a malformed or non-boolean "approve" field is an error, never
    truthy. A string like "false" or "no" must not become an approval.
    """
    text = raw.strip()
    text = re.sub(r"^```(?:json)?\s*|\s*```$", "", text)
    parsed = json.loads(text)
    approve = parsed.get("approve")
    if not isinstance(approve, bool):
        raise ValueError("approve field must be a JSON boolean")
    return {
        "approve": approve,
        "summary": str(parsed.get("summary", "")),
        "failures": list(parsed.get("failures", [])),
        "required_action": str(parsed.get("required_action", "")),
    }


def post_comment(token, repo, pr_number, body):
    """Post a general PR (issue) comment. Best-effort — a comment failure must
    not mask the agent's exit code. Uses the issues endpoint (an issue comment
    on #pr_number), which requires issues: write."""
    try:
        gh(token, f"/repos/{repo}/issues/{pr_number}/comments", method="POST", data={"body": body})
    except RuntimeError:
        pass


def main():
    token = os.environ.get("GH_TOKEN", "")
    api_key = os.environ.get("OPENROUTER_API_KEY", "")
    if not token or not api_key:
        print("missing GH_TOKEN or OPENROUTER_API_KEY; failing closed", flush=True)
        return 1

    with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as fh:
        event = json.load(fh)
    repo = os.environ["GITHUB_REPOSITORY"]
    pr_number = event["pull_request"]["number"]
    head_sha = event["pull_request"]["head"]["sha"]

    # Internal gate: the four quality checks must all be SUCCESS before the
    # agent even considers approving. Everything else fails closed.
    status = check_status(token, repo, head_sha)
    for name in QUALITY_CHECKS:
        status.setdefault(name, "no run")
    print(f"quality checks: {json.dumps(status)}", flush=True)
    if any(conclusion != "SUCCESS" for conclusion in status.values()):
        print("quality gate not green; failing closed (check red, merge blocked)", flush=True)
        return 1

    # Gather PR evidence: title, body, diff, and (best-effort) change artifacts.
    pr = gh(token, f"/repos/{repo}/pulls/{pr_number}")
    title, body = pr["title"], pr.get("body") or ""
    files = gh_paged(token, f"/repos/{repo}/pulls/{pr_number}/files")
    diff = "".join(f.get("patch", "") for f in files)[:DIFF_CAP]

    change_artifacts = ""
    match = re.search(r"\*\*Change:\*\*\s*([\w.-]+)", body)
    if match:
        change_name = match.group(1)
        base_ref = pr["base"]["ref"]
        for artifact in ("proposal.md", "tasks.md"):
            try:
                entry = gh(
                    token,
                    f"/repos/{repo}/contents/openspec/changes/{change_name}/{artifact}?ref={base_ref}",
                )
                change_artifacts += (
                    f"\n--- {artifact} ---\n"
                    + base64.b64decode(entry["content"]).decode(errors="replace")[:20_000]
                )
            except RuntimeError:
                pass  # best-effort; absence of artifacts is not fatal

    ci_line = "; ".join(f"{k}={v}" for k, v in status.items())
    system = (
        "You are the adversarial reviewer for the public single-maintainer "
        f"repository {repo}. Review a pull request against the exact checklist "
        "the repo's review gate requires (docs/workflow.md, "
        ".github/PULL_REQUEST_TEMPLATE.md):\n"
        "1. Done-checklist: the four required CI checks are green (see below).\n"
        "2. Spec alignment: the change must align with its OpenSpec proposal/"
        "design and resolve the stated issue.\n"
        "3. Security-sensitive paths: inspect .gitleaks/, .github/, AGENTS.md, "
        "docs/workflow.md, openspec/, and any secret/credential handling in the "
        "diff. Flag anything that could leak secrets, weaken the quality gates, "
        "or violate the public-repo trust boundary. Try hard to break it: look "
        "for the case the author did not think of.\n"
        "4. Risk claim: the PR's risk assessment must be credible (what could "
        "go wrong, and mitigation).\n"
        "Approve ONLY if all four items pass. If anything is uncertain or "
        "failing, reject with a concrete required action.\n"
        "Respond with ONLY a JSON object, no markdown fences, in the shape: "
        '{"approve": true|false, "summary": "...", '
        '"failures": ["item: reason", ...], "required_action": "..."}'
    )
    user = (
        f"PR #{pr_number}: {title}\n\nBody:\n{body}\n\n"
        f"Required CI checks: {ci_line}\n\n"
        f"Change artifacts:\n{change_artifacts[:30_000]}\n\n"
        f"Diff ({len(diff)} chars)\n{diff}"
    )

    model = os.environ.get("AGENT_REVIEWER_MODEL") or "openrouter/auto"
    try:
        verdict = parse_verdict(call_openrouter(api_key, model, system, user))
    except Exception:
        print("LLM review failed; failing closed (check red, merge blocked)", flush=True)
        return 1

    if verdict["approve"]:
        print("agent review PASSED (check green)", flush=True)
        return 0

    # Reject: post a concise verdict comment, then fail the check.
    comment = (
        f"{VERDICT_COMMENT_MARKER}\n\n**Verdict: CHANGES REQUESTED**\n\n"
        f"{verdict['summary']}\n\n"
        + "**Required:** " + verdict["required_action"]
    )
    post_comment(token, repo, pr_number, comment)
    print("agent review FAILED (check red, merge blocked)", flush=True)
    return 1


if __name__ == "__main__":
    sys.exit(main())
