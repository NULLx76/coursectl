#!/usr/bin/env bash
set -xe
# CONFIG
BRANCH="main"
repo="../../../embedded-systems-lab/template-project"
# END CONFIG
cd "$repo"
REMOTES=$(git remote | grep -v origin)

git checkout "$BRANCH"
git pull


while read -r target_remote; do
	git push "$target_remote" "$BRANCH"
done <<<"$REMOTES"
