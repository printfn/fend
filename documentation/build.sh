#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

pandoc --standalone \
    --output=index.html \
    --metadata-file=pandoc-metadata.yml \
    --lua-filter=include-code-files.lua \
    --lua-filter=include-files.lua \
    index.md

pandoc --standalone \
    --output=fend.1 \
    --lua-filter=include-code-files.lua \
    --lua-filter=include-files.lua \
    manpage.md
