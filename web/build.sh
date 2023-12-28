#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

(cd ../wasm && wasm-pack build --target no-modules --out-dir ../web/pkg)

convert -resize "128x128" ../icon/icon.svg fend-icon-128.png

mkdir -p documentation

(cd ../documentation && pandoc --standalone \
	--output=../web/documentation/index.html \
	--metadata-file=pandoc-metadata.yml \
	--lua-filter=include-code-files.lua \
	--lua-filter=include-files.lua \
	--lua-filter=add-header-ids.lua \
	index.md)
