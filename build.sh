#!/bin/bash

set -e  # Exit on error

# Get the project name from Cargo.toml
PROJECT_NAME=$(grep '^name' Cargo.toml | sed 's/name[[:space:]]*=[[:space:]]*//' | sed 's/"//g' | tr -d '[:space:]')
RELEASE_DIR="release/"

# Create release directory if it doesn't exist
mkdir -p "${RELEASE_DIR}"

# Function to build and copy binary for each target
build_and_copy() {
  local target=$1
  local extension=$2
  
  echo "Building for $target..."
  cargo build --release --target $target

  local binary_name="${PROJECT_NAME}${extension}"
  local output_name="${PROJECT_NAME}-${target}${extension}"

  echo "Copying binary for $target..."
  if [ -f "target/${target}/release/${binary_name}" ]; then
    cp "target/${target}/release/${binary_name}" "${RELEASE_DIR}${output_name}"
    echo "Created ${output_name}"
  else
    echo "Error: Binary not found for ${target}"
    return 1
  fi
}

# Build and copy binary for each target platform
build_and_copy "x86_64-apple-darwin" ""  # Intel Mac
build_and_copy "aarch64-apple-darwin" ""  # Apple Silicon Mac
# build_and_copy "x86_64-unknown-linux-gnu" ""  # Linux
build_and_copy "x86_64-pc-windows-gnu" ".exe"  # Windows

echo "Build and copy process completed."
echo "Created binaries:"
ls -l ${RELEASE_DIR}