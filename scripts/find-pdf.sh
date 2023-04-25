#!/usr/bin/env bash
set -xe
# CONFIG
BRANCH="main"
OUT=$(pwd)
repo="../../../embedded-systems-lab/template-project"
# END CONFIG
cd "$repo"
REMOTES=$(git remote | grep -v origin)

git fetch --all

while read -r target_remote; do
	git checkout "$target_remote"/"$BRANCH"
	mkdir -p "$OUT"/pdfs/"$target_remote"
	while read -r file; do
		if [ "$file" ]; then 
			cp "$file" "$OUT"/pdfs/"$target_remote"/
		fi
	done <<<$(rg --files --no-ignore-vcs | rg pdf)
done <<<"$REMOTES"
