#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

(cd ../wasm && wasm-pack build --target nodejs --out-dir pkg-nodejs)
echo "Renaming package to 'fend-wasm-nodejs'..."
jq "setpath([\"name\"]; \"fend-wasm-nodejs\")" ../wasm/pkg-nodejs/package.json >temp
mv temp ../wasm/pkg-nodejs/package.json

npm install
npm exec tsc
npm exec -- esbuild --bundle index.ts --outdir=dist --platform=node

rm -f lambda_package.zip
# don't include package.json because esbuild makes a CJS bundle that won't work if node finds `"type": "module"`
zip -j lambda_package.zip dist/index.js node_modules/fend-wasm-nodejs/fend_wasm_bg.wasm
