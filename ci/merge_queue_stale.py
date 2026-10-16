#!/usr/bin/env -S uv run --script
"""Cancel a merge-queue run the queue has already superseded (#974).

    merge_queue_stale.py [--repo OWNER/REPO] [--branch master]

Reads GITHUB_EVENT_NAME, GITHUB_EVENT_PATH and GITHUB_RUN_ID. When the
queue rebuilds an entry it starts a new merge_group run on a new ref and the
old run keeps going. This asks GraphQL for the head commits of the branch's
merge queue entries and, if this run's merge_group.head_sha is not among them,
runs `gh run cancel`. It is a no-op for every other event, and any error
(API, parsing) leaves the run alone: only a clean answer cancels.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys

QUERY = """query($owner:String!,$name:String!,$branch:String!){
  repository(owner:$owner,name:$name){
    mergeQueue(branch:$branch){ entries(first:100){ nodes{ headCommit{ oid } } } }
  }
}"""


def queue_heads(data: dict) -> set[str] | None:
    """Head oids of the queue entries in a GraphQL response, None if unusable."""
    try:
        queue = data["data"]["repository"]["mergeQueue"]
        if queue is None:
            return None
        return {n["headCommit"]["oid"] for n in queue["entries"]["nodes"] if n and n.get("headCommit")}
    except (KeyError, TypeError):
        return None


def is_stale(event_name: str, head_sha: str | None, heads: set[str] | None) -> bool:
    """True only for a merge_group run whose head is absent from a known queue."""
    if event_name != "merge_group" or not head_sha or heads is None:
        return False
    return head_sha not in heads


def fetch(repo: str, branch: str) -> dict | None:
    owner, name = repo.split("/", 1)
    r = subprocess.run(
        ["gh", "api", "graphql", "-f", f"query={QUERY}", "-F", f"owner={owner}", "-F", f"name={name}", "-F", f"branch={branch}"],
        capture_output=True, text=True,
    )
    if r.returncode != 0:
        print(f"merge queue lookup failed, leaving run alone: {r.stderr.strip()}")
        return None
    try:
        return json.loads(r.stdout)
    except ValueError:
        return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", ""))
    ap.add_argument("--branch", default="master")
    args = ap.parse_args()
    event = os.environ.get("GITHUB_EVENT_NAME", "")
    if event != "merge_group":
        return 0
    head_sha = None
    try:
        with open(os.environ["GITHUB_EVENT_PATH"]) as f:
            head_sha = json.load(f)["merge_group"]["head_sha"]
    except (KeyError, OSError, ValueError):
        pass
    data = fetch(args.repo, args.branch)
    heads = queue_heads(data) if data else None
    if not is_stale(event, head_sha, heads):
        print(f"merge-queue run {head_sha} is current (or unknown); continuing")
        return 0
    print(f"merge-queue run {head_sha} is no longer a queue entry head; cancelling")
    run_id = os.environ["GITHUB_RUN_ID"]
    subprocess.run(["gh", "run", "cancel", run_id, "--repo", args.repo], check=False)
    # Cancellation lands asynchronously; stop this job so nothing else starts.
    subprocess.run(["sleep", "60"], check=False)
    return 1


if __name__ == "__main__":
    sys.exit(main())
