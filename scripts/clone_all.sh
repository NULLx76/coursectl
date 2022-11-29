#!/usr/bin/env bash
set -xe

groups=$(cargo run -- projects 20197)
branch="2-performance"

mkdir -p student_solutions
cd student_solutions

while read -ra arr; do
  git clone --branch $branch "${arr[1]}"
done <<<"$groups"
