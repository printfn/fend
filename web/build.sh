#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

(cd ../wasm && wasm-pack build --target no-modules --out-dir ../web/pkg)

cp ../icon/fend-icon-128.png .

mkdir -p documentation

(cd ../documentation && pandoc --standalone \
    --output=../web/documentation/index.html \
    --metadata-file=pandoc-metadata.yml \
    --lua-filter=include-code-files.lua \
    --lua-filter=include-files.lua \
    index.md)
