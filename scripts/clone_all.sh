#!/usr/bin/env bash
set -xe

groups=$(cargo run -- projects 24389)
branch="main"
dir="student_solutions_$(date --iso-8601=hours)"

mkdir -p "$dir"
cd "$dir"

# TODO: Pull if dir exists
while read -ra arr; do
  git clone --branch $branch "${arr[1]}" || true
done <<<"$groups"
