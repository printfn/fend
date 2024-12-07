#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

(cd ../wasm && wasm-pack build --out-dir pkg-nodejs)
echo "Renaming package to 'fend-wasm-nodejs'..."
jq "setpath([\"name\"]; \"fend-wasm-nodejs\")" ../wasm/pkg-nodejs/package.json >temp
mv temp ../wasm/pkg-nodejs/package.json

npm install
npm exec tsc
node --experimental-strip-types esbuild.ts

rm -f lambda_package.zip

zip -j -r lambda_package.zip package.json dist/
