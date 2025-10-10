#!/usr/bin/env bash
# PMAT Roadmap Synchronization Script
# Syncs roadmap.yaml ticket states with GitHub Issues

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  PMAT Roadmap Synchronization${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check for gh CLI
if ! command -v gh >/dev/null 2>&1; then
    echo -e "${RED}❌ GitHub CLI (gh) not found${NC}"
    echo "Install with: sudo apt install gh  (or brew install gh on macOS)"
    exit 1
fi

# Check authentication
if ! gh auth status >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Not authenticated with GitHub${NC}"
    echo "Run: gh auth login"
    exit 1
fi

echo -e "${GREEN}✅ GitHub CLI authenticated${NC}"
echo ""

# Parse roadmap.yaml for Sprint 1 tickets
echo -e "${BLUE}📋 Analyzing roadmap.yaml for Sprint 1 tickets...${NC}"

TICKETS=(
    "DECY-001:Setup clang-sys integration and parse simple C function:critical:8"
    "DECY-002:Define HIR (High-level IR) structure for functions:critical:5"
    "DECY-003:Implement basic code generator for simple functions:high:8"
)

for ticket_info in "${TICKETS[@]}"; do
    IFS=':' read -r ticket_id title priority story_points <<< "$ticket_info"

    echo ""
    echo -e "${BLUE}Processing ${ticket_id}...${NC}"

    # Check if issue already exists
    EXISTING=$(gh issue list --label "ticket" --search "$ticket_id" --json number --jq '.[0].number' 2>/dev/null || echo "")

    if [ -n "$EXISTING" ]; then
        echo -e "${YELLOW}  Issue #${EXISTING} already exists for ${ticket_id}${NC}"
        echo -e "${YELLOW}  Skipping creation${NC}"
    else
        echo -e "${GREEN}  Creating GitHub issue for ${ticket_id}...${NC}"

        # Create issue body from roadmap
        BODY="## Ticket Information
**Ticket ID:** ${ticket_id}
**Sprint:** 1
**Story Points:** ${story_points}
**Priority:** ${priority}
**Type:** feature

## Description
${title}

See \`roadmap.yaml\` for complete specification including:
- Requirements
- Test requirements (unit, property, doctests, examples)
- Acceptance criteria
- RED-GREEN-REFACTOR workflow

## Definition of Done
- [ ] RED phase complete with failing tests
- [ ] GREEN phase complete with passing tests
- [ ] REFACTOR phase complete with quality gates met
- [ ] Coverage ≥ 80%
- [ ] 0 clippy warnings
- [ ] 0 SATD comments
- [ ] All tests passing
- [ ] Documentation complete
- [ ] CI pipeline green

## Reference
\`roadmap.yaml\` - Sprint 1 - ${ticket_id}
"

        # Create the issue
        ISSUE_NUM=$(gh issue create \
            --title "${ticket_id}: ${title}" \
            --body "$BODY" \
            --label "ticket,sprint-1,${priority}" \
            --assignee "@me" 2>/dev/null | grep -oP '#\K[0-9]+' || echo "")

        if [ -n "$ISSUE_NUM" ]; then
            echo -e "${GREEN}  ✅ Created issue #${ISSUE_NUM}${NC}"
        else
            echo -e "${YELLOW}  ⚠️  Could not create issue (may already exist)${NC}"
        fi
    fi
done

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Roadmap synchronization complete${NC}"
echo ""
echo "View issues: gh issue list --label sprint-1"
echo "View roadmap: cat roadmap.yaml"
echo ""
