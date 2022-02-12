#!/usr/bin/env bash
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
    added_lines=$(git --no-pager diff|grep -c '^+')
    if [[ "$added_lines" != "$1" ]]; then
        fail "Expected $1 lines + files to be different"
    fi
    removed_lines=$(git --no-pager diff|grep -c '^-')
    if [[ "$removed_lines" != "$1" ]]; then
        fail "Expected $1 lines + files to be different"
    fi
}

checkversion "$NEW_VERSION"

if [[ "$(git rev-parse --abbrev-ref HEAD)" != "main" ]]; then
    echo "Error: not on main branch"
fi

OLD_VERSION="$(cargo run -q -- version)"

confirm "Releasing update $OLD_VERSION -> $NEW_VERSION. Update the README file if necessary."

echo "Updating Cargo.lock" # also ensures the internet connection works
cargo update

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

manualstep "Add changelog to wiki/Home.md"
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
echo "'git commit -am \"Release version $NEW_VERSION\"'"
git commit -am "Release version $NEW_VERSION"
RELEASE_COMMIT_HASH=$(git rev-parse main)
git status
echo "'git push origin main'"
git push origin main

echo "Waiting for CI to start..."
sleep 5
GH_RUN_ID=$(gh run list -b main --json headSha,conclusion,name,status,url,workflowDatabaseId,event \
    | jq -r ".[] | select(.headSha == \"$RELEASE_COMMIT_HASH\") | .url" \
    | sed 's%https://github.com/printfn/fend/actions/runs/%%')
gh run watch --exit-status "$GH_RUN_ID"

echo "cargo publish for fend-core"
(cd core && cargo publish)
echo "Sleeping for 30 seconds to let crates.io update"
sleep 30
echo "cargo publish for fend"
(cd cli && cargo publish)
echo "Tag and push tag to GitHub"
git tag "v$NEW_VERSION"
git push --tags

echo "Building NPM package fend-wasm"
rm -rfv wasm/pkg
(cd wasm && wasm-pack build)
grep 'fend_wasm_bg.js' wasm/pkg/package.json
(cd wasm/pkg && npm publish --dry-run 2>&1)|grep "total files:"|grep 7
echo "Publishing npm package"
(cd wasm/pkg && npm publish)

echo "Building NPM package fend-wasm-web"
rm -rfv wasm/pkgweb
(cd wasm && wasm-pack build --target web --out-dir pkgweb)
echo "Renaming package to 'fend-wasm-web'"
sed 's/"name": "fend-wasm"/"name": "fend-wasm-web"/' wasm/pkgweb/package.json >temp
mv temp wasm/pkgweb/package.json
echo "Removing 'sideEffects: false'"
sed 's/"sideEffects": false//' wasm/pkgweb/package.json |
    sed 's/"types": "fend_wasm.d.ts",/"types": "fend_wasm.d.ts"/' >temp
mv temp wasm/pkgweb/package.json
(cd wasm/pkgweb && npm publish)

echo "Downloading Github artifacts..."
gh run download "$GH_RUN_ID" --dir artifacts

echo "Zipping artifacts"
# --junk-paths prevents directory names from being stored in the zip file,
# so the binary is stored at the top level
zip --junk-paths "artifacts/fend-$NEW_VERSION-linux-x64.zip" \
    "artifacts/fend-$NEW_VERSION-linux-x64/fend"
zip --junk-paths "artifacts/fend-$NEW_VERSION-macos-aarch64.zip" \
    "artifacts/fend-$NEW_VERSION-macos-aarch64/fend"
zip --junk-paths "artifacts/fend-$NEW_VERSION-macos-x64.zip" \
    "artifacts/fend-$NEW_VERSION-macos-x64/fend"
zip --junk-paths "artifacts/fend-$NEW_VERSION-windows-x64.zip" \
    "artifacts/fend-$NEW_VERSION-windows-x64/fend.exe"

echo "Creating GitHub release..."
gh release create "$NEW_VERSION" --title "Version $NEW_VERSION" \
    --draft \
    --notes "Changes in this version:\n\n* ..." \
    "artifacts/fend-$NEW_VERSION-linux-x64.zip" \
    "artifacts/fend-$NEW_VERSION-macos-aarch64.zip" \
    "artifacts/fend-$NEW_VERSION-macos-x64.zip" \
    "artifacts/fend-$NEW_VERSION-windows-x64.zip"

manualstep "Open https://github.com/printfn/fend/releases/tag/$NEW_VERSION, add changelog and publish"

echo "Deleting artifacts..."
rm -rfv artifacts

# AUR release
TMPDIR="$(mktemp -d)"
if [[ ! -e "$TMPDIR" ]]; then
    >&2 echo "Failed to create temp directory"
    exit 1
fi
echo "Switching to temporary directory $TMPDIR"
pushd "$TMPDIR" >/dev/null
git clone ssh://aur@aur.archlinux.org/fend.git
cd fend
git config user.name printfn
git config user.email printfn@users.noreply.github.com
curl -O "https://static.crates.io/crates/fend/fend-$NEW_VERSION.crate"
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
