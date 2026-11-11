#!/usr/bin/env python3
"""Plan, verify and publish the fork package family on crates.io.

`plan` lists family packages whose manifest version is not on crates.io yet.
`dry-run` packages and builds exactly those packages without credentials.
`publish` uploads them with Cargo. crates.io limits how quickly one publisher
can upload new versions of existing crates, and a release that bumps the whole
family exceeds that burst. When an upload is rate limited, `publish` waits for
the time crates.io names and retries with whatever is still missing.
"""

import argparse
import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
FAMILY = json.loads((Path(__file__).parent / "package-family.json").read_text())
API = "https://crates.io/api/v1/crates"
# crates.io asks API clients for an identifying User-Agent and one request a second.
USER_AGENT = "zakura-arti-publish (https://github.com/zakura-core/arti)"
RETRY_AFTER = re.compile(r"try again after (.+? GMT)")
# Each rate-limited round usually lands one more upload, so allow a full family.
ATTEMPTS = len(FAMILY) + 10


def is_published(name, version):
    """Returns whether crates.io already has `version` of `name`.

    Uses the API rather than the sparse index, because the index can lag a
    just-uploaded version and a second upload of it would fail.
    """
    request = urllib.request.Request(f"{API}/{name}/{version}", headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=30):
            return True
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise
    finally:
        time.sleep(1)


def unpublished(names):
    """Returns the subset of family package `names` whose manifest version is not on crates.io."""
    versions = {entry["name"]: entry["version"] for entry in FAMILY.values()}
    return [name for name in names if not is_published(name, versions[name])]


def cargo_publish(names, extra):
    """Runs one `cargo publish` for `names`, streaming its output, and returns (status, stderr)."""
    command = ["cargo", "publish", "--locked", *extra]
    for name in names:
        command += ["-p", name]
    process = subprocess.Popen(command, cwd=ROOT, stderr=subprocess.PIPE, text=True)
    captured = []
    for line in process.stderr:
        sys.stderr.write(line)
        captured.append(line)
    return process.wait(), "".join(captured)


def retry_delay(stderr):
    """Returns seconds to wait if `stderr` reports a crates.io rate limit, otherwise None."""
    match = RETRY_AFTER.search(stderr)
    if "429" not in stderr or not match:
        return None
    retry_at = parsedate_to_datetime(match.group(1))
    return max(0.0, (retry_at - datetime.now(timezone.utc)).total_seconds()) + 5


def publish(names):
    """Uploads `names` until each version is on crates.io, waiting out rate limits between rounds."""
    for attempt in range(1, ATTEMPTS + 1):
        names = unpublished(names)
        if not names:
            print("Every family version is on crates.io.", flush=True)
            return 0
        print(f"Attempt {attempt}: publishing {len(names)} packages.", flush=True)
        # The plan job already built these packages from this commit.
        status, stderr = cargo_publish(names, ["--no-verify"])
        if status == 0:
            continue
        delay = retry_delay(stderr)
        if delay is None:
            return status
        print(f"Rate limited by crates.io; retrying in {delay:.0f}s.", flush=True)
        time.sleep(delay)
    print("Stopped: packages are still unpublished after every attempt.", file=sys.stderr)
    return 1


def main():
    """Parses the command line and runs the requested step."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("step", choices=["plan", "dry-run", "publish"])
    parser.add_argument("--github-output", help="append `count=<n>` for the plan step to this file")
    args = parser.parse_args()

    names = unpublished([entry["name"] for entry in FAMILY.values()])
    if args.step == "plan":
        for name in names:
            print(f"Unpublished: {name}", flush=True)
        print(f"{len(names)} of {len(FAMILY)} family versions are not on crates.io.", flush=True)
        if args.github_output:
            with open(args.github_output, "a") as output:
                output.write(f"count={len(names)}\n")
        return 0
    if not names:
        print("Nothing to publish.", flush=True)
        return 0
    if args.step == "dry-run":
        return cargo_publish(names, ["--dry-run"])[0]
    return publish(names)


if __name__ == "__main__":
    sys.exit(main())
