#!/bin/bash

set -e  # Exit on error

# Get the project name from Cargo.toml
PROJECT_NAME=$(grep '^name' Cargo.toml | sed 's/name[[:space:]]*=[[:space:]]*//' | sed 's/"//g' | tr -d '[:space:]')
RELEASE_DIR="release/"

# Create release directory if it doesn't exist
mkdir -p "${RELEASE_DIR}"

# Function to build, copy binary, and create ZIP archive for each target
build_copy_and_zip() {
  local target=$1
  local extension=$2
  local use_cross=$3
  local use_windows_linker=$4
  
  echo "Building for $target..."

  # Create temporary .cargo/config.toml for Windows target
  if [ "$use_windows_linker" = true ]; then
    mkdir -p .cargo
    echo '[target.x86_64-pc-windows-gnu]' > .cargo/config.toml
    echo 'linker = "x86_64-w64-mingw32-gcc"' >> .cargo/config.toml
    echo "Temporary .cargo/config.toml created for Windows build"
  fi

  if [ "$use_cross" = true ]; then
    cross build --release --target $target
  else
    cargo build --release --target $target
  fi

  # Remove temporary .cargo/config.toml after Windows build
  if [ "$use_windows_linker" = true ]; then
    rm .cargo/config.toml
    echo "Temporary .cargo/config.toml removed"
  fi

  local binary_name="${PROJECT_NAME}${extension}"
  local output_name="${PROJECT_NAME}${extension}"
  local zip_name="${PROJECT_NAME}-${target}.zip"

  echo "Copying binary for $target..."
  if [ -f "target/${target}/release/${binary_name}" ]; then
    cp "target/${target}/release/${binary_name}" "${RELEASE_DIR}${output_name}"
    echo "Created ${output_name}"
    
    echo "Creating ZIP archive for $target..."
    (cd "${RELEASE_DIR}" && zip "${zip_name}" "${output_name}")
    echo "Created ${zip_name}"
    
    # Remove the uncompressed binary
    rm "${RELEASE_DIR}${output_name}"
  else
    echo "Error: Binary not found for ${target}"
    return 1
  fi
}

# Build, copy binary, and create ZIP archive for each target platform
# build_copy_and_zip "x86_64-apple-darwin" "" false false  # Intel Mac
# build_copy_and_zip "aarch64-apple-darwin" "" false false  # Apple Silicon Mac
build_copy_and_zip "x86_64-pc-windows-gnu" ".exe" false true  # Windows (with custom linker)
build_copy_and_zip "x86_64-unknown-linux-gnu" "" false false  # Linux (using cross)

echo "Build, copy, and ZIP process completed."
echo "Created ZIP archives:"
ls -l ${RELEASE_DIR}