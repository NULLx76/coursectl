#!/usr/bin/env bash
set -xe
repo="$HOME/src/software-systems/student-template-repository"
cd "$repo"

# CONFIG
REMOTES=$(git remote | grep --invert-match origin)
SOURCE_BRANCH=1-concurrency-update
TARGET_BRANCH=1-concurrency
TITLE="Fix example 3"
DESCRIPTION="We made a mistake in Example 3 which can cause issues when trying to implement binary mode. This MR should address those issues"
# END CONFIG

while read -r target_remote; do
	git push -o merge_request.create -o merge_request.target="$TARGET_BRANCH" -o merge_request.title="$TITLE" -o merge_request.description="$DESCRIPTION" -o merge_request.remove_source_branch="true" "$target_remote" "$SOURCE_BRANCH"
done <<<"$REMOTES"
