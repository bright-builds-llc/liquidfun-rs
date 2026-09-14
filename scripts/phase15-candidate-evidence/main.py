"""Candidate evidence CLI. Collection records confer no release authority."""

import argparse
import sys
import tomllib

from common import ROOT, WORKFLOWS, require
from dispatch import dispatch, reconcile
from payloads import names
from retention import collect, validate_retained


def check(_args):
    with (ROOT / "reference/release/required-evidence.toml").open("rb") as stream:
        registry = tomllib.load(stream)
    require(len(registry["evidence"]) == 19, "closed registry cardinality differs")
    artifacts = [name for workflow in WORKFLOWS for name in names(workflow, 1, "a" * 40)]
    require(len(set(artifacts)) == len(artifacts) == 21, "closed artifact cardinality differs")
    print("candidate evidence contract: 21 artifacts / 19 registry entries")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check").set_defaults(operation=check)
    for name, operation in (("dispatch", dispatch), ("reconcile", reconcile),
                            ("collect", collect), ("validate-retained", validate_retained)):
        command = commands.add_parser(name)
        command.set_defaults(operation=operation)
        command.add_argument("--candidate", required=True)
        command.add_argument("--attempt-root", required=True,
                             help="ignored repository-relative target/ path; preserve failed attempts")
        if name in ("dispatch", "collect"):
            command.add_argument("--workflow", required=True)
            command.add_argument("--ref", required=True)
        if name == "dispatch":
            command.add_argument("--input", action="append", default=[])
        if name == "collect":
            command.add_argument("--run-id", required=True)
            command.add_argument("--run-attempt", required=True)
        if name == "validate-retained":
            command.add_argument("--destination", help="new target/ destination for restored 21 artifact directories")
            command.add_argument("--validator-bash", default="bash", help="Bash >=4; GNU find/jq/SHA256 tools on PATH")
    args = parser.parse_args()
    args.operation(args)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, TypeError) as error:
        print(f"candidate evidence: {error}", file=sys.stderr)
        sys.exit(1)
