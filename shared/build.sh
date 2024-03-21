#!/usr/bin/env bash
set -e

# Check if jq is installed
if ! [ -x "$(command -v jq)" ]; then
    echo "jq is not installed" >& 2
    exit 1
fi

# thanks to zrzka for the following idea to merge both builds
# see https://github.com/rustwasm/wasm-pack/issues/313#issuecomment-441163923

DBG=
if [[ $@ == *"--debug"* ]]; then
    DBG=--debug
fi

export CARGO_TARGET_DIR=target-wasm

if [[ ( $@ == *"--node"* ) || ( $@ != *"--browser"* ) ]]; then
    if [ -d "pkg-node" ]; then
        rm -rf pkg-node
    fi

    # Build
    wasm-pack build $DBG -t nodejs -d pkg-node -- --target-dir target-wasm

    # Get the package name
    PKG_NAME=$(jq -r .name pkg-node/package.json | sed 's/\-/_/g')
fi

if [[ ( $@ == *"--browser"* ) || ( $@ != *"--node"* ) ]]; then
    # Clean previous packages
    if [ -d "pkg" ]; then
        rm -rf pkg-browser
    fi

    # Build
    wasm-pack build $DBG -t browser -d pkg-browser -- --target-dir target-wasm

    # Get the package name
    PKG_NAME=$(jq -r .name pkg-browser/package.json | sed 's/\-/_/g')
fi

mkdir -p pkg

# Merge nodejs & browser packages
cp "pkg-node/${PKG_NAME}.js" "pkg/${PKG_NAME}.js"
cp "pkg-node/${PKG_NAME}_bg.wasm" "pkg/${PKG_NAME}.wasm"
cp "pkg-node/${PKG_NAME}_bg.wasm.d.ts" "pkg/${PKG_NAME}.wasm.d.ts"
cp "pkg-browser/${PKG_NAME}.js" "pkg/${PKG_NAME}.mjs"
cp "pkg-browser/${PKG_NAME}_bg.js" "pkg/${PKG_NAME}_bg.js"
cp "pkg-browser/${PKG_NAME}_bg.wasm" "pkg/${PKG_NAME}_bg.wasm"
cp "pkg-browser/${PKG_NAME}_bg.wasm.d.ts" "pkg/${PKG_NAME}_bg.wasm.d.ts"

sed 's/_bg.wasm/.wasm/g' -i "pkg/${PKG_NAME}.js"

# if [ ! -f pkg/package.json ]; then
    cat pkg-browser/package.json \
        | jq ".sideEffects = [\"${PKG_NAME}.mjs\"]" \
        | jq ".main = \"${PKG_NAME}.js\"" \
        | jq ".files += [\"${PKG_NAME}.mjs\"]" \
        | jq ".files += [\"${PKG_NAME}_bg.js\"]" \
        | jq ".files += [\"${PKG_NAME}.wasm\"]" \
        | jq ".module = \"${PKG_NAME}.mjs\"" > pkg/package.json

    # cat pkg/package.json
    # cat pkg-node/package.json
# fi

rm -rf pkg-node
rm -rf pkg-browser
