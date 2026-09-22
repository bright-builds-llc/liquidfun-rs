#!/usr/bin/env bash
# Commit the Dam Break README SVG only when the generated files changed.
set -euo pipefail

svg_path="docs/assets/demos/dam-break-10s.svg"
readme_path="README.md"
github_actions_name="github-actions[bot]"
github_actions_email="41898282+github-actions[bot]@users.noreply.github.com"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ "${GITHUB_ACTIONS:-}" != "true" ]]; then
	printf '%s\n' "Refusing to publish the Dam Break SVG outside GitHub Actions."
	exit 1
fi

git add -- "$readme_path" "$svg_path"
if git diff --cached --quiet -- "$readme_path" "$svg_path"; then
	printf '%s\n' "Dam Break SVG and README are unchanged; not committing."
	exit 0
fi

git fetch origin main
if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
	printf '%s\n' "origin/main moved after checkout; not committing."
	exit 0
fi

git -c "user.name=${github_actions_name}" -c "user.email=${github_actions_email}" \
	commit -m "$(cat <<'EOF'
chore: update the Dam Break animated SVG

Regenerate the 10 second default Dam Break SVG and its README section.
EOF
)" -- "$readme_path" "$svg_path"

set +e
push_output="$(git push origin HEAD:main 2>&1)"
push_status=$?
set -e
printf '%s\n' "$push_output"
if [[ "$push_status" -eq 0 ]]; then
	exit 0
fi

if printf '%s\n' "$push_output" | grep -Eiq 'non-fast-forward|fetch first|tip of your current branch is behind'; then
	printf '%s\n' "Push rejected because main moved; a newer run will publish the SVG."
	exit 0
fi

printf '%s\n' "Push failed. The job needs contents: write, and the repository Actions setting must allow that token to push. If main rejects github-actions[bot], set BRIGHT_BUILDS_PUSH_TOKEN to a token that can push to main."
exit "$push_status"
