#!/usr/bin/env bash
set -xe
# CONFIG
BRANCH="main"
repo="../../software-fundamentals/individual-assignment-template"
# END CONFIG
cd "$repo"
REMOTES=$(git remote | grep -v origin)

git fetch --all
git checkout "$BRANCH"
git pull

while read -r target_remote; do
	git fetch "$target_remote"
	git switch -c "$target_remote-$BRANCH" "$target_remote/$BRANCH"
	git pull origin "$BRANCH" --no-edit
	git push "$target_remote" HEAD:"$BRANCH"
	git checkout "$BRANCH"
	git branch -D "$target_remote-$BRANCH"
done <<<"$REMOTES"
