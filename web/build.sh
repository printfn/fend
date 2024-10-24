#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Ubuntu 24.04 doesn't ship with ImageMagick v7
if ! type magick &>/dev/null; then
	shopt -s expand_aliases # https://unix.stackexchange.com/a/1498
	alias magick=convert
fi

npm ci
npm run check

rm -rf ../wasm/pkg-fend-web
(cd ../wasm && wasm-pack build --target web --out-dir pkg-fend-web)

magick ../icon/icon.svg -resize "128x128" public/fend-icon-128.png

mkdir -p public/documentation

(cd ../documentation && pandoc --standalone \
	--output=../web/public/documentation/index.html \
	--metadata-file=pandoc-metadata.yml \
	--lua-filter=include-code-files.lua \
	--lua-filter=include-files.lua \
	--lua-filter=add-header-ids.lua \
	index.md)

npm run build
