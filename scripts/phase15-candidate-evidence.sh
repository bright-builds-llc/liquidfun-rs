#!/usr/bin/env bash
set -euo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=scripts/phase15-candidate-evidence/dispatch.sh
source "$script_directory/phase15-candidate-evidence/dispatch.sh"
# shellcheck source=scripts/phase15-candidate-evidence/retention.sh
source "$script_directory/phase15-candidate-evidence/retention.sh"

case "${1:-}" in
dispatch | reconcile) candidate_dispatch "$@" ;;
collect | validate-retained) candidate_retention "$@" ;;
*) exec python3 -B "$script_directory/phase15-candidate-evidence/main.py" "$@" ;;
esac
