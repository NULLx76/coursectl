#!/usr/bin/env bash
set -xe
repo="$HOME/src/software-systems/student-template-repository"
cd "$repo"
git pull

# CONFIG
REMOTES=$(git remote | grep -v origin)
BRANCH="2-performance"
# END CONFIG

while read -r target_remote; do
	git push "$target_remote" "$BRANCH"
done <<<"$REMOTES"
