#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

a() {
    aws --no-cli-pager --region ap-southeast-2 "$@"
}

echo "Updating function configuration..."

# # Warning:
#
# The `update-function-configuration` and `update-function-code` commands
# print all environment variables, including the Telegram Bot API token,
# so we redirect the output to /dev/null.

a lambda update-function-configuration \
    --function-name fend-telegram-bot \
    --environment "Variables={TELEGRAM_BOT_API_TOKEN=$TELEGRAM_BOT_API_TOKEN}" >/dev/null

a lambda wait function-updated-v2 --function-name fend-telegram-bot

echo "Updating function code..."

a lambda update-function-code \
    --function-name fend-telegram-bot \
    --zip-file fileb://lambda_package.zip >/dev/null

a lambda wait function-updated-v2 --function-name fend-telegram-bot
