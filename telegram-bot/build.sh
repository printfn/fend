#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

rm -rfv ../wasm/pkg
(cd ../wasm && wasm-pack build)

npm ci
npm exec tsc
node esbuild.ts

rm -f lambda_package.zip

zip -j -r lambda_package.zip package.json dist/
