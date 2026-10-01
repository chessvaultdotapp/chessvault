#!/usr/bin/env bash
set -euo pipefail

# Run from the repository root with the compared head checked out.
base=$1
head=$2
event=$3

if [[ "$event" == pull_request ]]; then
  base=$(git merge-base "$base" "$head")
elif [[ "$base" =~ ^0+$ ]]; then
  # A new branch has no before commit; treat its files as additions.
  base=$(git hash-object -t tree /dev/null)
fi

# Disable rename detection so moved workflows are checked as new files.
# Capture the diff first so a Git failure cannot silently become an empty list.
changed=$(mktemp)
trap 'rm -f "$changed"' EXIT
git diff --name-only --no-renames --diff-filter=ACMRT -z "$base" "$head" -- \
  .github/workflows/ .github/actions/ > "$changed"

workflows=()
while IFS= read -r -d '' path; do
  case "$path" in
    .github/workflows/*.yml|.github/workflows/*.yaml)
      workflows+=("$path")
      ;;
    .github/actions/*)
      printf 'Not checked by actionlint (action implementation): %s\n' "$path"
      ;;
  esac
done < "$changed"

if (( ${#workflows[@]} == 0 )); then
  echo 'No added or modified workflow YAML files to check.'
  exit 0
fi

# Keep the checks identical regardless of optional analyzers on the runner.
go tool actionlint -shellcheck= -pyflakes= "${workflows[@]}"
