#!/usr/bin/env bash
set -euo pipefail

repo_url="${TASKWARRIOR_TESTDATA_REPO_URL:-https://github.com/kdheepak/taskwarrior-testdata}"
ref="${TASKWARRIOR_TESTDATA_REF:-149099b69457404e20e73b428db00c7e88aca8d3}"
testdata_dir="${TASKWARRIOR_TESTDATA_DIR:-tests/data}"

rm -rf "$testdata_dir"
mkdir -p "$(dirname "$testdata_dir")"

git init -q "$testdata_dir"
git -C "$testdata_dir" remote add origin "$repo_url"
git -C "$testdata_dir" fetch --depth 1 origin "$ref"
git -C "$testdata_dir" checkout --detach FETCH_HEAD

echo "Prepared taskwarrior-testdata at $testdata_dir pinned to $ref"
