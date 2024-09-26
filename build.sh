#!/bin/bash

set -e  # エラーが発生した時点でスクリプトを終了

# Get the project name from Cargo.toml
PROJECT_NAME=$(grep '^name' Cargo.toml | sed 's/name[[:space:]]*=[[:space:]]*//' | sed 's/"//g' | tr -d '[:space:]')

# Function to build and create ZIP for each target
build_and_zip() {
  local target=$1
  local extension=$2
  
  echo "Building for $target..."
  cargo build --release --target $target

  local binary_name="${PROJECT_NAME}${extension}"
  local zip_name="${PROJECT_NAME}-${target}.zip"

  echo "Creating ZIP for $target..."
  if [ -f "target/${target}/release/${binary_name}" ]; then
    # Create a temporary directory
    local temp_dir=$(mktemp -d)
    
    # Copy the binary
    cp "target/${target}/release/${binary_name}" "${temp_dir}/"
    
    # Copy additional files and directories
    cp -R assets "${temp_dir}/" || echo "Warning: 'assets' directory not found"
    cp .env.example "${temp_dir}/" || echo "Warning: '.env.example' file not found"
    cp config.sepolia.toml "${temp_dir}/" || echo "Warning: 'config.sepolia.toml' file not found"
    cp README.md "${temp_dir}/" || echo "Warning: 'README.md' file not found"
    
    # Create the ZIP file
    (cd "${temp_dir}" && zip -r "${zip_name}" .)
    
    # Move the ZIP file to the project root
    mv "${temp_dir}/${zip_name}" .
    
    # Remove the temporary directory
    rm -rf "${temp_dir}"
    
    echo "Created ${zip_name}"
  else
    echo "Error: Binary not found for ${target}"
    return 1
  fi
}

# Build and create ZIP for each target platform
build_and_zip "x86_64-apple-darwin" ""  # Intel Mac
build_and_zip "aarch64-apple-darwin" ""  # Apple Silicon Mac
build_and_zip "x86_64-unknown-linux-gnu" ""  # Linux
# build_and_zip "x86_64-pc-windows-msvc" ".exe"  # Windows

echo "Build and ZIP process completed."
echo "Created ZIP files:"
ls -l *.zip