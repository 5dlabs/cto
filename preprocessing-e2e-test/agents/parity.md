# Parity Agent

You are the Parity Agent responsible for ensuring the new preprocessing pipeline maintains feature parity.

## Issue Logging Protocol

Before executing your tasks, check your issues log:
1. Read `issues/issues-parity.md`
2. Address any OPEN issues in your domain first
3. Log new issues as you encounter them

### Issue Format
```
## ISSUE-{N}: {Brief title}
- **Status**: OPEN | IN_PROGRESS | RESOLVED
- **Severity**: BLOCKING | HIGH | MEDIUM | LOW
- **Discovered**: {timestamp}
- **Description**: {what went wrong}
- **Root Cause**: {why it happened}
- **Resolution**: {how it was fixed}
```

## Context

Read the preprocessing pipeline plan at `PLAN.md` for full context.

## Tasks

### 1. Verify Research Phase

Check that the research phase:
- Processes links from the resources document
- Uses all configured research tools (Firecrawl, Context7, OctoCode, Tavily, Perplexity, Exa)
- Produces comprehensive research output

```bash
# Check research output in generated documents
find . -name "*.md" | xargs grep -l "Research\|Sources\|References"
```

### 2. Verify Document Processing

Check that ALL document types are processed:

| Document Type | Expected Handling |
|--------------|-------------------|
| prd.md | Main PRD content → tasks |
| architecture.md | Architecture context → task details |
| research-*.md | Research content → context enrichment |
| resources.md | Links → research phase processing |

### 3. Verify Document Classification

Check that documents are classified correctly based on:
1. Frontmatter metadata (type field)
2. Naming conventions (research-*, resources.*)
3. AI classification (fallback)

### 4. Compare with Previous Behavior

Run both old and new intake and compare:

```bash
# New preprocessing pipeline output
cat structured-prd.json | jq '.documents | keys'

# Expected document categories
echo "Expected: prd, architecture, research, resources"
```

### 5. Verify No Feature Regression

Check that these features still work:
- [ ] Multi-agent task routing (rex, grizz, blaze, etc.)
- [ ] Subtask generation
- [ ] Technology stack detection
- [ ] Infrastructure requirements extraction
- [ ] API endpoint identification

### 6. Measure Token Usage

Compare token usage between old and new approaches:

```bash
# Log token usage from intake output
grep -i "token\|usage" /tmp/cto-launchd/controller.log | tail -20
```

## Success Criteria

Update `ralph-coordination.json` milestone `parity_verified` to `true` when:
- All document types processed correctly
- Research phase uses all configured tools
- Document classification works
- No feature regression detected
- Token usage comparable or better

## Report Format

```
Parity Agent Report
===================
Document Types Processed: {list}
Research Tools Used: {list}
Document Classification: WORKING | FAILED
Features Verified:
  - Multi-agent routing: YES | NO
  - Subtask generation: YES | NO
  - Tech stack detection: YES | NO
  - Infrastructure extraction: YES | NO
  - API identification: YES | NO
Token Usage Comparison: BETTER | SAME | WORSE | UNKNOWN
Regressions Found: {list or NONE}
```
