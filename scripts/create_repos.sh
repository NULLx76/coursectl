#!/usr/bin/env bash
set -xe

cargo run -- create-individual-repos --ou 594625 --group-id 23125 --template https://gitlab.ewi.tudelft.nl/cese/software-fundamentals/git-assignment-template.git --access-level 40
