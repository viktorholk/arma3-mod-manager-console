
#!/bin/bash

# Project name
PROJECT_NAME="arma3-mod-manager-console"

# Define targets
TARGETS=(
    "aarch64-apple-darwin"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-gnu"
)

# Function to build for a specific target
build_for_target() {
    local target=$1
    rustup target add $target

    echo "Building for target ${target} in release mode..."

    if [[ "$target" == *"-linux-gnu" ]]; then
        cross build --target "${target}" --release
    else
        cargo build --target "${target}" --release
    fi

    # Path to the binary
    local binary_path="target/${target}/release/${PROJECT_NAME}"

    # Check if the binary exists
    if [[ -f "${binary_path}" ]]; then
        # Make the binary executable
        chmod +x "${binary_path}"

        # Create the zip file containing only the binary
        (cd "target/${target}/release" && zip "${PROJECT_NAME}-${target}-release.zip" "${PROJECT_NAME}")

        # Move the zip file to the project root
        mv "target/${target}/release/${PROJECT_NAME}-${target}-release.zip" .
    else
        echo "Binary not found for target ${target} in release mode."
    fi
}

# Main script logic
for target in "${TARGETS[@]}"; do
    build_for_target "${target}"
done

echo "Release builds and zips created successfully!"

