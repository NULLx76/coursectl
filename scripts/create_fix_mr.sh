#!/usr/bin/env bash
set -xe
repo="../../../embedded-systems-lab/template-project"
cd "$repo"

# CONFIG
REMOTES=$(git remote | grep --invert-match origin)
OWN_USER_ID=1027
SOURCE_BRANCH=template-update-0
TARGET_BRANCH=main
TITLE="Update quadrupel to v2"
DESCRIPTION="This fixes some bugs in the library causing occasional panics during boot."
# END CONFIG

while read -r target_remote; do
	git push -o merge_request.create -o merge_request.unassign="$OWN_USER_ID" -o merge_request.target="$TARGET_BRANCH" -o merge_request.title="$TITLE" -o merge_request.description="$DESCRIPTION" -o merge_request.remove_source_branch="true" "$target_remote" "$SOURCE_BRANCH"
done <<<"$REMOTES"
