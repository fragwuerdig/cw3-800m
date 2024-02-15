#! /bin/bash

# This script is used to verify the SHA256 hash of the on-chain wasm
# file against the build artifact from the code of this repository 
# Assuming following installations:
# - jq 
# - shasum
# - curl
# - base64
# - docker

set -e

curl -sSX GET "https://terra-classic-lcd.publicnode.com:443/cosmwasm/wasm/v1/code/8508" -H  "accept: application/json" | jq -r '.data' > download_base64.wasm
base64 -d download_base64.wasm > download_raw.wasm

./build.sh
SHA_DOWNLOAD=$(shasum -a 256 download_raw.wasm | awk '{print $1}')
SHA_LOCAL=$(shasum -a 256 artifacts/cw3_fixed_multisig.wasm | awk '{print $1}')

echo "Downloaded SHA:     $SHA_DOWNLOAD"
echo "Local Built SHA:    $SHA_LOCAL"

