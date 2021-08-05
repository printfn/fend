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

gitdiff() {
    # checks the expected number of lines + files are different
    added_lines=$(git --no-pager diff|grep '^+'|wc -l)
    if [ $added_lines -ne $1 ]; then
        fail "Expected $1 lines + files to be different"
    fi
    removed_lines=$(git --no-pager diff|grep '^-'|wc -l)
    if [ $removed_lines -ne $1 ]; then
        fail "Expected $1 lines + files to be different"
    fi
}

checkversion "$NEW_VERSION"

OLD_VERSION="$(cargo run -q -- version)"

confirm "Releasing update $OLD_VERSION -> $NEW_VERSION. Update the README file if necessary."
echo "Checking if the README files are in sync..."
diff README.md cli/README.md
diff README.md core/README.md
echo "Running cargo fmt..."
cargo fmt -- --check

echo "Making sure the git repository is clean..."
# from https://stackoverflow.com/a/5143914
if ! git diff-index --quiet HEAD --; then
    fail "The local repository has uncommitted changes"
fi

echo "Bumping version numbers..."

# version number in fend-core
sed "s/^version = \"$OLD_VERSION\"$/version = \"$NEW_VERSION\"/" core/Cargo.toml >temp
mv temp core/Cargo.toml

# fend-core docs attr
sed "s|https://docs.rs/fend-core/$OLD_VERSION|https://docs.rs/fend-core/$NEW_VERSION|" core/src/lib.rs >temp
mv temp core/src/lib.rs

# fend-core get_version_as_str()
sed "s/\"$OLD_VERSION\"/\"$NEW_VERSION\"/" core/src/lib.rs >temp
mv temp core/src/lib.rs

# fend cli TOML x2
sed "s/^version = \"$OLD_VERSION\"$/version = \"$NEW_VERSION\"/" cli/Cargo.toml >temp
mv temp cli/Cargo.toml

# wasm TOML
sed "s/^version = \"$OLD_VERSION\"$/version = \"$NEW_VERSION\"/" wasm/Cargo.toml >temp
mv temp wasm/Cargo.toml

# fend web initialisation
sed "s/release: \"fend@$OLD_VERSION\"/release: \"fend@$NEW_VERSION\"/" web/index.html >temp
mv temp web/index.html

# wiki
sed "s/version of fend is \`$OLD_VERSION\`/version of fend is \`$NEW_VERSION\`/" wiki/Home.md >temp
mv temp wiki/Home.md

gitdiff 14

manualstep "Add changelog to wiki"
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
grep 'fend_wasm_bg.js' wasm/pkg/package.json
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
echo test|shasum -a 512 -|grep "^0e3e75234abc68f4378a86b3f4b32"
HASH=$(shasum -a 512 "fend-$NEW_VERSION.crate" | grep -o '[a-f0-9]\{128\}')
echo "Hash: $HASH"
rm "fend-$NEW_VERSION.crate"
sed "s/$OLD_VERSION/$NEW_VERSION/g" .SRCINFO|sed "s/[a-f0-9]\{128\}/$HASH/" >.SRCINFO_NEW
sed "s/$OLD_VERSION/$NEW_VERSION/" PKGBUILD|sed "s/[a-f0-9]\{128\}/$HASH/" >PKGBUILD_NEW
mv .SRCINFO_NEW .SRCINFO
mv PKGBUILD_NEW PKGBUILD
gitdiff 7 # 5 lines in 2 files
git commit -am "fend $OLD_VERSION -> $NEW_VERSION"
git --no-pager log --pretty=full HEAD~..|grep '^Author: printfn <printfn@users.noreply.github.com>$'
git --no-pager log --pretty=full HEAD~..|grep '^Commit: printfn <printfn@users.noreply.github.com>$'
git push origin master
popd >/dev/null
rm -rf "$TMPDIR"
