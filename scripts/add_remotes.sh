#!/usr/bin/env bash
set -xe
data=$(cargo run -- projects 23180)
repo="../../software-fundamentals/individual-assignment-template"
cd "$repo"
while read -r line; do
  # shellcheck disable=SC2086
  git remote add ${line} || true
done <<<"$data"
