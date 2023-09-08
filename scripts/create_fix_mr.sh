#!/usr/bin/env bash
set -xe
repo="../../software-fundamentals/individual-assignment-template"
cd "$repo"

# CONFIG
REMOTES=$(git remote | grep --invert-match origin)
OWN_USER_ID=1027
SOURCE_BRANCH=minor-fix-1
TARGET_BRANCH=main
TITLE="Minor Fix 1 for Template"
DESCRIPTION="There was a required method call missing in the template for actually generating the HTML. This PR adds that."
# END CONFIG

while read -r target_remote; do
	git push -o merge_request.create -o merge_request.unassign="$OWN_USER_ID" -o merge_request.target="$TARGET_BRANCH" -o merge_request.title="$TITLE" -o merge_request.description="$DESCRIPTION" -o merge_request.remove_source_branch="true" "$target_remote" "$SOURCE_BRANCH"
done <<<"$REMOTES"
