#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

if [[ $# -eq 0 ]] ; then
    echo "Please specify new version number, e.g. '0.1.0'"
    exit 0
fi
NEW_VERSION=$1

fail() {
    echo "$1"
    exit 1
}

checkversion() {
    echo "$1" | grep "^[0-9]\+\.[0-9]\+\.[0-9]\+$" >/dev/null || fail "Invalid version"
}

confirm() {
    echo "$1"
    read -r -p "Press enter to confirm, or Ctrl-C to cancel"
    echo
}

manualstep() {
    echo
    echo "Manual step:"
    confirm "$1"
}

checkversion "$NEW_VERSION"

OLD_VERSION="$(cargo run -q -- version)"

confirm "Releasing update $OLD_VERSION -> $NEW_VERSION"
echo "Checking if the README files are in sync..."
diff README.md cli/README.md
diff README.md core/README.md
echo "Running cargo fmt..."
cargo fmt
manualstep "Update README"
manualstep "Bump version number in these places:
* fend-core TOML,
* fend-core docs attr,
* fend-core get_version_as_str(),
* fend cli TOML,
* fend cli TOML version requirement for fend-core
* fend wasm TOML
* fend web initialisation
* fend wiki

Add changelog to wiki"
echo "Building and running tests..."
cargo clippy --workspace --all-targets --all-features
cargo build
cargo run -- version
cargo test --all
echo "'cargo run -- version'"
cargo run -q -- version
cargo run -q -- version | grep "$NEW_VERSION" || fail "cargo run -- version returned the wrong version"
echo "Committing..."
git add -A
git --no-pager diff --cached
confirm "'git commit -am \"Release version $NEW_VERSION\"'"
git commit -am "Release version $NEW_VERSION"
git status
confirm "'git push'"
git push
manualstep "Ensure CI passes"
confirm "cargo publish for fend-core"
(cd core && cargo publish)
echo "Sleeping for 30 seconds to let crates.io update"
sleep 30
confirm "cargo publish for fend"
(cd cli && cargo publish)
confirm "Tag and push tag to GitHub"
git tag "v$NEW_VERSION"
git push --tags
confirm "Build NPM package"
(cd wasm && wasm-pack build)
confirm 'Opening vim to add "fend_wasm_bg.js" to package.json'
vim wasm/pkg/package.json
(cd wasm/pkg && npm publish --dry-run)
confirm "Publish npm package"
(cd wasm/pkg && npm publish)
manualstep "Create GitHub release (including changelog):
  * Download artifacts from 'https://github.com/printfn/fend/actions'
  * Go to 'https://github.com/printfn/fend/releases/new'
  * Title: Version $NEW_VERSION
  * Text:
Changes in this version:

* ..."

# AUR release
TMPDIR="$(mktemp -d)"
if [ ! -e "$TMPDIR" ]; then
    >&2 echo "Failed to create temp directory"
    exit 1
fi
echo "Switching to temporary directory $TMPDIR"
pushd "$TMPDIR" >/dev/null
git clone ssh://aur@aur.archlinux.org/fend.git
cd fend
git config user.name printfn
git config user.email printfn@users.noreply.github.com
curl -o "fend-$NEW_VERSION.crate" "https://static.crates.io/crates/fend/fend-$NEW_VERSION.crate"
HASH=$(shasum -a 512 "fend-$NEW_VERSION.crate" | grep -o '[a-f0-9]\{128\}')
echo "Hash: $HASH"
rm "fend-$NEW_VERSION.crate"
sed "s/$OLD_VERSION/$NEW_VERSION/g" .SRCINFO|sed "s/[a-f0-9]\{128\}/$HASH/" >.SRCINFO_NEW
sed "s/$OLD_VERSION/$NEW_VERSION/" PKGBUILD|sed "s/[a-f0-9]\{128\}/$HASH/" >PKGBUILD_NEW
mv .SRCINFO_NEW .SRCINFO
mv PKGBUILD_NEW PKGBUILD
git --no-pager diff --cached
manualstep "Check the diff: 5 lines should have changed"
git commit -am "fend $OLD_VERSION -> $NEW_VERSION"
git --no-pager log
manualstep "Check the log"
git push origin
popd >/dev/null
rm -rf "$TMPDIR"
