#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

(cd ../wasm && wasm-pack build --target nodejs --out-dir pkg-nodejs)
echo "Renaming package to 'fend-wasm-nodejs'..."
jq "setpath([\"name\"]; \"fend-wasm-nodejs\")" ../wasm/pkg-nodejs/package.json >temp
mv temp ../wasm/pkg-nodejs/package.json

npm install

rm -f lambda_package.zip
zip -r lambda_package.zip \
    node_modules/ index.js \
    package.json package-lock.json
