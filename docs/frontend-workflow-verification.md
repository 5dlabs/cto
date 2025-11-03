# Frontend Workflow Verification

## Task Detection Test

### Test Case 1: Frontend Task with React
```json
{
  "id": 1,
  "title": "Build user dashboard with Material-UI",
  "agentType": "frontend",
  "language": "typescript",
  "framework": "react"
}
```

**Expected Routing:**
- ✅ Detects as frontend task
- ✅ Routes to `frontend-agent` (Blaze)
- ✅ Uses `frontend-cli` and `frontend-model`
- ✅ Passes `typescript` + `react` to agents

### Test Case 2: Content-Based Detection
```json
{
  "id": 2,
  "title": "Implement React component for user profile",
  "description": "Create a Material-UI based user profile component"
}
```

**Expected Routing:**
- ✅ Auto-detects "react", "component", "material-ui" keywords
- ✅ Sets agentType = "frontend"
- ✅ Routes to Blaze

## Agent Behavior Test

### Blaze (Implementation)
**Expected:**
- ✅ Has access to shadcn MCP tools
- ✅ Can use context7 for library docs
- ✅ Creates React/TypeScript code
- ✅ Creates PR with implementation

### Cleo (Quality)
**Expected:**
- ✅ Receives TASK_LANGUAGE=typescript
- ✅ Receives TASK_FRAMEWORK=react
- ✅ Runs TypeScript/React quality checks:
  - ESLint for TypeScript
  - React Testing Library tests
  - TypeScript compilation
- ✅ Posts single comment
- ✅ Adds ready-for-qa label

### Cipher (Security)
**Expected:**
- ✅ Runs npm audit
- ✅ Checks for exposed secrets
- ✅ Validates dependencies
- ❓ **NOTE**: Doesn't receive language context (acceptable)

### Tess (QA/Testing)
**Expected:**
- ✅ Receives TASK_LANGUAGE=typescript
- ✅ Receives TASK_FRAMEWORK=react
- ✅ Gets Playwright instructions in prompt
- ✅ Sets up Playwright: `npm init playwright@latest`
- ✅ Creates E2E tests in tests/ directory
- ✅ Runs tests: `npx playwright test`
- ✅ Takes screenshots (homepage.png, interactions.png, etc.)
- ✅ Posts screenshots to PR with gh pr comment
- ✅ Commits screenshots to PR branch
- ✅ Posts approval review

## Screenshot Posting Flow

**Tess Instructions** (from container-tess.sh.hbs lines 1624-1672):

```bash
# Create comment with screenshots
cat > /tmp/screenshot-comment.md <<'EOF'
## 🎨 Frontend Testing Results
**E2E Tests:** PASSED ✅
### Screenshots
EOF

# Upload each screenshot
for screenshot in screenshots/*.png; do
  FILENAME=$(basename "$screenshot")
  echo "![$FILENAME](./$screenshot)" >> /tmp/screenshot-comment.md
done

# Post to PR
gh pr comment ${PR_NUMBER} --body-file /tmp/screenshot-comment.md

# Commit to PR branch
git add screenshots/
git commit -m "test: add Playwright E2E test screenshots"
git push origin HEAD
```

## Validation Steps

1. **Create frontend task** with agentType="frontend"
2. **Trigger play workflow** for that task
3. **Verify Blaze** receives task (check CodeRun)
4. **Verify Cleo** gets typescript+react context
5. **Verify Tess** runs Playwright and posts screenshots
6. **Check PR** for:
   - Screenshot comment from Tess
   - screenshots/ directory committed
   - Cleo single quality comment
   - Tess approval review

## Known Gaps

### 1. Cipher Frontend Context
- **Issue**: Cipher doesn't receive TASK_LANGUAGE/TASK_FRAMEWORK
- **Impact**: Low - security checks mostly language-agnostic
- **Fix**: Could pass env vars in play-workflow-template.yaml

### 2. Cleo Frontend Enhancements
- **Current**: Basic React standards
- **Could Add**:
  - ESLint React plugin requirements
  - React Hooks rules validation
  - TypeScript strict mode enforcement

## Recommendation

The frontend workflow is **90% complete**:
- ✅ Task detection works
- ✅ Blaze has proper tools
- ✅ Tess has Playwright instructions
- ✅ Screenshot posting is documented

**Next Step**: Run end-to-end test with a frontend task to verify Tess actually posts screenshots.




