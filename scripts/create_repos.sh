#!/usr/bin/env bash
set -xe

cargo run -- create-individual-repos --group-id 23125 --template https://gitlab.ewi.tudelft.nl/cese/software-fundamentals/git-assignment-template.git --student-list ./classlist.json --access-level 40
