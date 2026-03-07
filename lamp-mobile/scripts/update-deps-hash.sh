#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

arch=$(nix eval --impure --expr 'builtins.currentSystem' 2>/dev/null | tr -d '"')
target=".#packages.${arch}.mavenRepo"

echo "Resolving Maven dependencies for ${arch}..."
echo "Downloads JARs/AARs from Maven Central into the Nix store."
echo ""

# Build the FOD. First run fails with hash mismatch, showing the correct hash.
output=$(nix build "$target" 2>&1) && {
    echo "Maven repo already up to date."
    exit 0
}

# Extract the actual content hash from the error output.
got_hash=$(echo "$output" | grep -oP 'got:\s+\K\S+' | head -1)

if [ -z "$got_hash" ]; then
    echo "Could not extract hash. Full output:"
    echo "$output"
    exit 1
fi

echo "Computed hash: $got_hash"

# Patch flake.nix with the real hash.
sed -i "s|outputHash = \"sha256-[A-Za-z0-9+/=]*\"|outputHash = \"$got_hash\"|" flake.nix

echo "Updated flake.nix"
echo ""
echo "Verifying build..."
nix build "$target" --no-link && {
    echo ""
    echo "Done. Run: nix develop"
} || {
    echo "Verification failed. Check flake.nix."
    exit 1
}
