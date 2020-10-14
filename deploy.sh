#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

if [[ $# -eq 0 ]] ; then
    echo "Please specify new version number, e.g. '0.1.0'"
    exit 0
fi
VERSION=$1

fail() {
    echo "$1"
    exit 1
}

echo $VERSION | grep "^\d\+\.\d\+\.\d\+$" >/dev/null || fail "Invalid version"

confirm() {
    echo "$1"
    read -p "Press enter to confirm, or Ctrl-C to cancel"
    echo
}

manualstep() {
    echo
    echo "Manual step:"
    confirm "$1"
}

confirm "Releasing version $VERSION"
echo "Running cargo fmt..."
cargo fmt
manualstep "Update changelog"
manualstep "Update README"
manualstep "Bump version number in these places:
fend-core TOML,
fend-core docs attr,
fend-core get_version(),
fend-core get_extended_version() function,
fend TOML,
fend print_version(),
fend TOML version requirement for fend-core"
echo "Building and running tests..."
touch core/src/lib.rs
cargo clippy --workspace --all-targets --all-features
cargo build
cargo run -- version
cargo test --all
echo "'cargo run -- version'"
cargo run -q -- version
cargo run -q -- version | grep $VERSION || fail "cargo run -- version returned the wrong version"
echo "Committing..."
git add -A
git --no-pager diff --cached
confirm "'git commit -am \"Release version $VERSION\"'"
git commit -am "Release version $VERSION"
git status
confirm "'git push'"
git push
manualstep "Ensure CI passes"
echo "'(cd core && cargo publish --dry-run)'"
(cd core && cargo publish --dry-run)
confirm "cargo publish for fend-core"
(cd core && cargo publish)
echo "Sleeping for 30 seconds to let crates.io update"
sleep 30
echo "'cargo publish --dry-run'"
cargo publish --dry-run
confirm "cargo publish for fend"
cargo publish
confirm "Tag and push tag to GitHub"
git tag "v$VERSION"
git push --tags
manualstep "Create GitHub release (including changelog). Download and rename artifacts."
manualstep "Update manual: https://github.com/printfn/fend-rs/wiki"
manualstep "Update AUR package, see https://github.com/printfn/fend-aur/wiki"
