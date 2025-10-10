#!/bin/bash
# PMAT: Validate documentation links
# Ensures all local file links in markdown files exist

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 Validating documentation links..."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BROKEN_LINKS=0
CHECKED_FILES=0
TEMP_FILE=$(mktemp)

# Find all markdown files
for md_file in $(find "$PROJECT_ROOT" -name "*.md" -not -path "*/target/*" -not -path "*/.git/*" -not -path "*/node_modules/*"); do
    CHECKED_FILES=$((CHECKED_FILES + 1))
    md_dir="$(dirname "$md_file")"

    # Extract all markdown links: [text](url)
    if grep -oE '\]\([^)]+\)' "$md_file" > /dev/null 2>&1; then
        grep -oE '\]\([^)]+\)' "$md_file" | sed 's/](\([^)]*\))/\1/' | while read -r link; do
            # Skip external links
            if [[ "$link" =~ ^https?:// ]]; then
                continue
            fi

            # Skip anchors only
            if [[ "$link" =~ ^# ]]; then
                continue
            fi

            # Remove anchor fragments
            file_path="${link%%#*}"

            # Resolve path relative to the markdown file's directory
            if [[ "$file_path" == /* ]]; then
                # Absolute path from project root
                full_path="$PROJECT_ROOT/$file_path"
            else
                # Relative path
                full_path="$md_dir/$file_path"
            fi

            # Check if file exists
            if [ ! -e "$full_path" ]; then
                echo -e "${RED}✗ Broken link in $(basename "$md_file"):${NC}"
                echo -e "  Link: ${YELLOW}$link${NC}"
                echo -e "  Expected: ${YELLOW}$full_path${NC}"
                echo ""
                echo "1" >> "$TEMP_FILE"
            fi
        done
    fi
done

# Count broken links from temp file
if [ -f "$TEMP_FILE" ]; then
    BROKEN_LINKS=$(wc -l < "$TEMP_FILE")
    rm "$TEMP_FILE"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$BROKEN_LINKS" -eq 0 ]; then
    echo -e "${GREEN}✅ All documentation links valid${NC}"
    echo "   Checked $CHECKED_FILES markdown files"
    exit 0
else
    echo -e "${RED}✗ Found $BROKEN_LINKS broken link(s) across $CHECKED_FILES files${NC}"
    exit 1
fi
