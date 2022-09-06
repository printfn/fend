#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

pandoc --standalone \
    --output=index.html \
    --metadata-file=pandoc-metadata.yml \
    index.md ../CHANGELOG.md