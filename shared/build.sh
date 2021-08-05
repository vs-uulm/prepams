#!/usr/bin/env bash
set -e

# Check if jq is installed
if ! [ -x "$(command -v jq)" ]; then
    echo "jq is not installed" >& 2
    exit 1
fi

# Clean previous packages
if [ -d "pkg" ]; then
    rm -rf pkg
fi

if [ -d "pkg-node" ]; then
    rm -rf pkg-node
fi

DBG=

if [[ $@ == *"--debug"* ]]; then
    DBG=--debug
fi

# thanks to zrzka for the following idea to merge both builds
# see https://github.com/rustwasm/wasm-pack/issues/313#issuecomment-441163923

# Build for both targets
wasm-pack build $DBG -t browser -d pkg
wasm-pack build $DBG -t nodejs -d pkg-node

# Get the package name
PKG_NAME=$(jq -r .name pkg/package.json | sed 's/\-/_/g')

# Merge nodejs & browser packages
cp "pkg/${PKG_NAME}.js" "pkg/${PKG_NAME}.mjs"
cp "pkg-node/${PKG_NAME}.js" "pkg/${PKG_NAME}.js"
cp "pkg-node/${PKG_NAME}_bg.wasm" "pkg/${PKG_NAME}.wasm"

sed 's/_bg.wasm/.wasm/g' -i "pkg/${PKG_NAME}.js"

cat pkg-node/package.json \
    | jq ".sideEffects = false" \
    | jq ".files += [\"${PKG_NAME}.mjs\"]" \
    | jq ".files += [\"${PKG_NAME}_bg.js\"]" \
    | jq ".files += [\"${PKG_NAME}.wasm\"]" \
    | jq ".module = \"${PKG_NAME}.mjs\"" > pkg/package.json

cat pkg/package.json

rm -rf pkg-node
