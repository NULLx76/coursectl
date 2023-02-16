#!/usr/bin/env bash
set -xe
data=$(cargo run -- projects 21201)
repo="../../../embedded-systems-lab/template-project"
cd "$repo"
while read -r line; do
  # shellcheck disable=SC2086
  git remote add ${line} || true
done <<<"$data"
