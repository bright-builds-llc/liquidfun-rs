#!/usr/bin/env bash
# Commit README scene SVGs only when the generated files changed.
set -euo pipefail

readme_path="README.md"
svg_dir="docs/assets/readme"
github_actions_name="github-actions[bot]"
github_actions_email="41898282+github-actions[bot]@users.noreply.github.com"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if [[ "${GITHUB_ACTIONS:-}" != "true" ]]; then
	printf '%s\n' "Refusing to publish README scene SVGs outside GitHub Actions."
	exit 1
fi

git add -A -- "$readme_path" "$svg_dir"
if git diff --cached --quiet -- "$readme_path" "$svg_dir"; then
	printf '%s\n' "README scene SVGs and gallery are unchanged; not committing."
	exit 0
fi

git fetch origin main
if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
	printf '%s\n' "origin/main moved after checkout; not committing."
	exit 0
fi

git -c "user.name=${github_actions_name}" -c "user.email=${github_actions_email}" \
	commit -m "$(cat <<'EOF'
chore: update README scene SVGs

Regenerate the 10 second scene SVGs and the README demo gallery.
EOF
)" -- "$readme_path" "$svg_dir"

set +e
push_output="$(git push origin HEAD:main 2>&1)"
push_status=$?
set -e
printf '%s\n' "$push_output"
if [[ "$push_status" -eq 0 ]]; then
	exit 0
fi

if printf '%s\n' "$push_output" | grep -Eiq 'non-fast-forward|fetch first|tip of your current branch is behind'; then
	printf '%s\n' "Push rejected because main moved; a newer run will publish the SVGs."
	exit 0
fi

printf '%s\n' "Push failed. The job needs contents: write, and the repository Actions setting must allow that token to push. If main rejects github-actions[bot], set BRIGHT_BUILDS_PUSH_TOKEN to a token that can push to main."
exit "$push_status"
