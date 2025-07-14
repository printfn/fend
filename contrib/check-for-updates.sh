#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# This script checks for updates to any of fend's dependencies
cargo update

cargo metadata --format-version 1 --no-deps \
	| jq -r '.packages[].dependencies[] | select(.name | contains("fend") | not) | .name + " " + (.req | sub("\\^"; ""))' \
	| while read -r line
	do
		IFS=" " read -r -a words <<< "$line" # https://www.shellcheck.net/wiki/SC2206
		dep=${words[0]}
		current_version=${words[1]}
		latest_version="$(curl -sL "https://crates.io/api/v1/crates/$dep" | jq -r .crate.max_stable_version)"
		if [[ "$current_version" != "$latest_version" ]]
		then
			echo "Update available for $dep: $current_version -> $latest_version"
		fi
	done

(cd web && npx npm-check-updates -u && npm update)
(cd telegram-bot && npx npm-check-updates -u && npm update)

current_wix="$(jq -r .tools.wix.version < windows-wix/.config/dotnet-tools.json)"
# https://learn.microsoft.com/en-us/nuget/api/registration-base-url-resource
latest_wix="$(curl -sL "$(
	curl -sL https://api.nuget.org/v3/index.json \
		| jq -r '.resources[] | select(."@type" == "RegistrationsBaseUrl/3.6.0") | ."@id" + "wix/index.json"')" \
	| gzip -d \
	| jq '.items[0].items[].catalogEntry.version | select (. | contains("-") | not)' \
	| jq -sr '. | last')"

if [[ "$latest_wix" != "$current_wix" ]]
then
	echo "Update available for wix: $current_wix -> $latest_wix"
	echo "Also check for updates to the wix extensions"
fi
