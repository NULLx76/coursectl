#!/usr/bin/env bash
set -xe

cargo run -- create-individual-repos --ou 594625 --group-id 23180 --template https://gitlab.ewi.tudelft.nl/cese/software-fundamentals/individual-assignment-template.git
