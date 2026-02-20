#!/usr/bin/env bash
# Validates that docs/schema/CHANGELOG.md documents every MemoryType variant
# defined in src/memory/record.rs. Fails CI if any variant is undocumented.
set -euo pipefail

RECORD_FILE="src/memory/record.rs"
CHANGELOG="docs/schema/CHANGELOG.md"

if [ ! -f "$RECORD_FILE" ]; then
    echo "FAIL: $RECORD_FILE not found"
    exit 1
fi

if [ ! -f "$CHANGELOG" ]; then
    echo "FAIL: $CHANGELOG not found — run scripts/gen-schema-changelog.sh to create it"
    exit 1
fi

echo "Checking schema changelog coverage against $RECORD_FILE..."

# Extract MemoryType variant names (lines matching /    Variant,/ inside the enum block)
in_enum=0
missing=()
while IFS= read -r line; do
    if echo "$line" | grep -q "^pub enum MemoryType"; then
        in_enum=1
        continue
    fi
    if [ $in_enum -eq 1 ] && echo "$line" | grep -q "^}"; then
        in_enum=0
        break
    fi
    if [ $in_enum -eq 1 ]; then
        variant=$(echo "$line" | grep -oP '^\s+\K[A-Z][a-zA-Z]+(?=,)' || true)
        if [ -n "$variant" ]; then
            lower=$(echo "$variant" | tr '[:upper:]' '[:lower:]')
            if ! grep -qi "$lower\|$variant" "$CHANGELOG"; then
                missing+=("$variant")
            fi
        fi
    fi
done < "$RECORD_FILE"

if [ ${#missing[@]} -gt 0 ]; then
    echo "FAIL: These MemoryType variants are not documented in $CHANGELOG:"
    for m in "${missing[@]}"; do
        echo "  - $m"
    done
    echo ""
    echo "Update $CHANGELOG with a new version entry covering these types."
    exit 1
fi

echo "OK: All MemoryType variants are documented in the schema changelog."
