#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

pandoc index.md ../CHANGELOG.md -s -o index.html