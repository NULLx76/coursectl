#!/usr/bin/env bash
set -xe
data=$(cargo run -- projects 20197)
repo="$HOME/src/software-systems/student-template-repository"
cd "$repo"
while read -r line; do
  # shellcheck disable=SC2086
  git remote add ${line}
done <<< "$data"
