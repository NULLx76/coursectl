#!/usr/bin/env bash
set -xe
data=$(cargo run -- projects 24388)
repo="/home/vivian/src/cese/real-time-systems/assignment-a-template"
cd "$repo"
while read -r line; do
  # shellcheck disable=SC2086
  git remote add ${line} || true
done <<<"$data"
