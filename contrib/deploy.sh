#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cd ..

USAGE="Usage: ./deploy.sh [flags] <version>

<version> should be the new version number to release, e.g. 0.1.0

Flags:
-h  --help            show this help screen"

NEW_VERSION=""
while [[ "$#" != 0 ]]; do
    arg="$1"
    if [[ "$arg" == "-h" || "$arg" == "--help" ]]; then
        echo "$USAGE"
        exit
    elif [[ "$arg" =~ ^- ]]; then
        echo "error: unknown option '$arg'" >&2
        exit 1
    elif [[ "$NEW_VERSION" == "" ]]; then
        NEW_VERSION="$arg"
    else
        echo "error: too many arguments" >&2
        exit 1
    fi
    shift
done

if [[ "$NEW_VERSION" == "" ]] ; then
    echo "$USAGE" >&2
    exit 1
fi

fail() {
    echo "$1"
    exit 1
}

checkversion() {
    echo "$1" | grep "^[0-9]\+\.[0-9]\+\.[0-9]\+$" >/dev/null \
        || fail "Invalid version"
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
    local gitdir="$1"
    local expected_add_count="$2"
    local expected_del_count="$3"
    # checks the expected number of lines + files are different
    local added_lines
    added_lines="$(git -C "$gitdir" --no-pager diff|grep -c '^+')"
    if [[ "$added_lines" != "$expected_add_count" ]]; then
        fail "Expected $expected_add_count lines+files to be different (+)"
    fi
    local removed_lines
    removed_lines="$(git -C "$gitdir" --no-pager diff|grep -c '^-')"
    if [[ "$removed_lines" != "$expected_del_count" ]]; then
        fail "Expected $expected_del_count lines+files to be different (-)"
    fi
}

checkversion "$NEW_VERSION"

current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$current_branch" != "main" ]]; then
    echo "Error: not on main branch"
fi

OLD_VERSION="$(cargo run --package fend --quiet -- version)"

if ! command -v "wasm-pack"; then
    fail "Please install wasm-pack"
fi

confirm "Releasing update $OLD_VERSION -> $NEW_VERSION. \
Update the README file and other documentation if necessary."

echo "Updating Cargo.lock" # also ensures the internet connection works
cargo update

echo "Running cargo fmt..."
cargo fmt -- --check

echo "Making sure we are logged in to npm..."
npm whoami

PATH="$HOME/.cargo/bin:$PATH"
echo "Ensure that we are using Rustup"
if [[ ! "$(which rustc)" =~ .cargo/bin/rustc$ ]]; then
    fail "Using $(which rustc) which does not seem to be from Rustup"
fi

echo "Making sure the git repository is clean..."
# from https://stackoverflow.com/a/5143914
git update-index --refresh &>/dev/null || true
if ! git diff-index --quiet HEAD --; then
    fail "The local repository has uncommitted changes"
fi

echo "Bumping version numbers..."

# version number in fend-core
sed "s/^version = \"$OLD_VERSION\"$/version = \"$NEW_VERSION\"/" \
    core/Cargo.toml >temp
mv temp core/Cargo.toml

# fend cli TOML x2
sed "s/^version = \"$OLD_VERSION\"$/version = \"$NEW_VERSION\"/" \
    cli/Cargo.toml >temp
mv temp cli/Cargo.toml

# wasm TOML
sed "s/^version = \"$OLD_VERSION\"$/version = \"$NEW_VERSION\"/" \
    wasm/Cargo.toml >temp
mv temp wasm/Cargo.toml

gitdiff "" 7 7

manualstep "Add changelog to CHANGELOG.md"

echo "Extracted changelog:"
CHANGELOG=$(tr "\n" "\1" <CHANGELOG.md \
    | grep --text -o "### v$NEW_VERSION .*### v$OLD_VERSION" \
    | tr "\1" "\n" \
    | tail +3 \
    | sed "\$d" \
    | sed "\$d")
echo "$CHANGELOG"

manualstep "Make sure this is the correct changelog"

echo "Building and running tests..."
cargo clippy --workspace --all-targets --all-features
cargo build
cargo run -- version
cargo test --all
echo "'cargo run -- version'"
cargo run -q -- version
cargo run -q -- version | grep "$NEW_VERSION" \
    || fail "cargo run -- version returned the wrong version"
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
GH_RUN_ID=$(gh run list -b main --json databaseId,headSha \
    | jq ".[] | select(.headSha == \"$RELEASE_COMMIT_HASH\") | .databaseId")

manualstep "Wait for GitHub CI to pass"

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
echo "Renaming package to 'fend-wasm-web' and removing 'sideEffects: false'..."
jq "setpath([\"name\"]; \"fend-wasm-web\") | del(.sideEffects)" \
    wasm/pkgweb/package.json >temp
mv temp wasm/pkgweb/package.json
(cd wasm/pkgweb && npm publish)

echo "Building NPM package fend-wasm-nodejs"
rm -rfv wasm/pkg-nodejs
(cd wasm && wasm-pack build --target nodejs --out-dir pkg-nodejs)
echo "Renaming package to 'fend-wasm-nodejs'..."
jq "setpath([\"name\"]; \"fend-wasm-nodejs\")" wasm/pkg-nodejs/package.json >temp
mv temp wasm/pkg-nodejs/package.json
(cd wasm/pkg-nodejs && npm publish --dry-run 2>&1)|grep "total files:"|grep 6
echo "Publishing npm package"
(cd wasm/pkg-nodejs && npm publish)

TMPDIR="$(mktemp -d)"
if [[ ! -d "$TMPDIR" ]]; then
    >&2 echo "Failed to create temp directory"
    exit 1
fi
echo "Created temporary directory $TMPDIR"

manualstep "Ensure GitHub CI has finished and all artifacts are available"

echo "Downloading Github artifacts..."
gh run download "$GH_RUN_ID" --dir "$TMPDIR/artifacts"

echo "Zipping artifacts..."
# --junk-paths prevents directory names from being stored in the zip file,
# so the binary is stored at the top level
zip --junk-paths "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-x64.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-x64/fend"
zip --junk-paths "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-aarch64-gnu.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-aarch64-gnu/fend"
zip --junk-paths "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-armv7-gnueabihf.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-armv7-gnueabihf/fend"
zip --junk-paths "$TMPDIR/artifacts/fend-$NEW_VERSION-macos-aarch64.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-macos-aarch64/fend"
zip --junk-paths "$TMPDIR/artifacts/fend-$NEW_VERSION-macos-x64.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-macos-x64/fend"
zip --junk-paths "$TMPDIR/artifacts/fend-$NEW_VERSION-windows-x64-exe.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-windows-x64-exe/fend.exe"
cp "$TMPDIR/artifacts/fend-windows-x64-msix/fend-windows-x64.msix" \
    "$TMPDIR/artifacts/fend-windows-x64.msix"
cp "$TMPDIR/artifacts/fend-windows-x64-msi/fend-windows-x64.msi" \
    "$TMPDIR/artifacts/fend-windows-x64.msi"

echo "Creating GitHub release..."
CHANGELOG2=$'Changes in this version:\n\n'"$CHANGELOG"
gh release --repo printfn/fend \
    create "v$NEW_VERSION" --title "Version $NEW_VERSION" \
    --notes "$CHANGELOG2" \
    --draft \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-x64.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-aarch64-gnu.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-linux-armv7-gnueabihf.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-macos-aarch64.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-macos-x64.zip" \
    "$TMPDIR/artifacts/fend-$NEW_VERSION-windows-x64-exe.zip" \
    "$TMPDIR/artifacts/fend-windows-x64.msix" \
    "$TMPDIR/artifacts/fend-windows-x64.msi"

manualstep "Open https://github.com/printfn/fend/releases and check \
that the new release is correct. If it is, go ahead and publish it."

# AUR release
git clone ssh://aur@aur.archlinux.org/fend.git "$TMPDIR/fend-aur"
git -C "$TMPDIR/fend-aur" config user.name printfn
git -C "$TMPDIR/fend-aur" config user.email printfn@users.noreply.github.com
echo test|shasum -a 512 -|grep "^0e3e75234abc68f4378a86b3f4b32" >/dev/null
HASH=$(curl -L -o - "https://static.crates.io/crates/fend/fend-$NEW_VERSION.crate" \
    | shasum -a 512 - \
    | grep -o '[a-f0-9]\{128\}')
echo "Hash: $HASH"
sed "s/$OLD_VERSION/$NEW_VERSION/g" "$TMPDIR/fend-aur/.SRCINFO" \
    | sed "s/[a-f0-9]\{128\}/$HASH/" >"$TMPDIR/fend-aur/.SRCINFO_NEW"
sed "s/$OLD_VERSION/$NEW_VERSION/" "$TMPDIR/fend-aur/PKGBUILD" \
    | sed "s/[a-f0-9]\{128\}/$HASH/" >"$TMPDIR/fend-aur/PKGBUILD_NEW"
mv "$TMPDIR/fend-aur/.SRCINFO_NEW" "$TMPDIR/fend-aur/.SRCINFO"
mv "$TMPDIR/fend-aur/PKGBUILD_NEW" "$TMPDIR/fend-aur/PKGBUILD"
gitdiff "$TMPDIR/fend-aur" 7 7 # 5 lines in 2 files
git -C "$TMPDIR/fend-aur" commit -am "fend $OLD_VERSION -> $NEW_VERSION"
git -C "$TMPDIR/fend-aur" --no-pager log --pretty=full HEAD~.. \
    | grep '^Author: printfn <printfn@users.noreply.github.com>$'
git -C "$TMPDIR/fend-aur" --no-pager log --pretty=full HEAD~.. \
    | grep '^Commit: printfn <printfn@users.noreply.github.com>$'
git -C "$TMPDIR/fend-aur" push origin master

git clone git@github.com:printfn/homebrew-fend "$TMPDIR/homebrew-fend"
git -C "$TMPDIR/homebrew-fend" config user.name printfn
git -C "$TMPDIR/homebrew-fend" config user.email printfn@users.noreply.github.com
echo test|shasum -a 256 -|grep "^f2ca1bb6c7e907d06dafe4687e579" >/dev/null
HASH=$(curl -L -o - "https://github.com/printfn/fend/archive/refs/tags/v$NEW_VERSION.tar.gz" \
    | shasum -a 256 - \
    | grep -o '[a-f0-9]\{64\}')
URL_START="https://github.com/printfn/fend/archive/refs/tags"
URL2_START="https://github.com/printfn/homebrew-fend/releases/download"
sed "s%${URL_START}/v$OLD_VERSION.tar.gz%${URL_START}/v$NEW_VERSION.tar.gz%" \
    "$TMPDIR/homebrew-fend/Formula/fend.rb" \
    | sed "s/^  sha256 \"[0-9a-f]\{64\}\"/  sha256 \"$HASH\"/" \
    | sed "s%root_url \"$URL2_START/v$OLD_VERSION\"%root_url \"$URL2_START/v$NEW_VERSION\"%" \
    | grep -v "sha256 cellar:" \
    | grep -v "^    rebuild " \
    >"$TMPDIR/homebrew-fend/Formula/fend-new.rb"
mv "$TMPDIR/homebrew-fend/Formula/fend-new.rb" "$TMPDIR/homebrew-fend/Formula/fend.rb"
gitdiff "$TMPDIR/homebrew-fend" 4 6
git -C "$TMPDIR/homebrew-fend" commit -am "fend $OLD_VERSION -> $NEW_VERSION"
git -C "$TMPDIR/homebrew-fend" --no-pager log --pretty=full HEAD~.. \
    | grep '^Author: printfn <printfn@users.noreply.github.com>$'
git -C "$TMPDIR/homebrew-fend" --no-pager log --pretty=full HEAD~.. \
    | grep '^Commit: printfn <printfn@users.noreply.github.com>$'
git -C "$TMPDIR/homebrew-fend" push origin main
brew uninstall printfn/fend/fend
brew update
brew install rust
brew install --build-bottle --verbose printfn/fend/fend
brew bottle --no-rebuild printfn/fend/fend
mv "fend--$NEW_VERSION.arm64_monterey.bottle.tar.gz" \
    "fend-$NEW_VERSION.arm64_monterey.bottle.tar.gz"
git -C "$TMPDIR/homebrew-fend" tag "v$NEW_VERSION"
git -C "$TMPDIR/homebrew-fend" push --tags origin main
gh release --repo printfn/homebrew-fend \
    create "v$NEW_VERSION" --title "Version $NEW_VERSION" \
    --notes "v$NEW_VERSION" \
    "fend-$NEW_VERSION.arm64_monterey.bottle.tar.gz"
manualstep "Add bottle info to $TMPDIR/homebrew-fend/Formula/fend.rb"
#gitdiff "$TMPDIR/homebrew-fend" 2 1
git -C "$TMPDIR/homebrew-fend" commit -am \
    "v$NEW_VERSION: Add reference to fend-$NEW_VERSION.arm64_monterey.bottle.tar.gz"
git -C "$TMPDIR/homebrew-fend" push origin main
brew uninstall printfn/fend/fend
brew update
brew install printfn/fend/fend
manualstep "Make sure the bottle was used"

rm "fend-$NEW_VERSION.arm64_monterey.bottle.tar.gz"
rm -rf "$TMPDIR"
