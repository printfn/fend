#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

npm install

echo "exports.TELEGRAM_API_TOKEN = '$TELEGRAM_BOT_API_TOKEN'" >telegram_api_token.js

rm -f lambda_package.zip
zip -r lambda_package.zip \
    node_modules/ index.js telegram_api_token.js \
    package.json package-lock.json

aws --no-cli-pager --region ap-southeast-2 lambda update-function-code \
    --function-name fend-telegram-bot \
    --zip-file fileb://lambda_package.zip
