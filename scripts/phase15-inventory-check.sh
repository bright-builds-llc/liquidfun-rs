#!/usr/bin/env bash
set -euo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
exec python3 -B "$script_directory/phase15-candidate-evidence/docs_ci.py" --native-inventory "$@"
