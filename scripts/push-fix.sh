#!/usr/bin/env bash
set -xe
# CONFIG
BRANCH="main"
repo="../../software-systems/student-template-repository"
# END CONFIG
cd "$repo"
REMOTES=$(git remote | grep -v origin)

git fetch --all
git checkout "$BRANCH"
git pull
git config pull.rebase false

while read -r target_remote; do
	git fetch "$target_remote"
	git branch -D "$target_remote-$BRANCH" || true # remove old branch if it exists
	git switch -c "$target_remote-$BRANCH" "$target_remote/$BRANCH"
	git pull origin "$BRANCH" --no-edit
	git push "$target_remote" HEAD:"$BRANCH"
	git checkout "$BRANCH"
	git branch -D "$target_remote-$BRANCH"
done <<<"$REMOTES"
