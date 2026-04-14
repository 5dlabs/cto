# Sigma-1 Pipeline Execution Command

This document contains the exact command that should be run to execute the intake pipeline once credentials are available.

## Prerequisites

Before running this command, ensure:
1. 1Password credentials are available OR
2. Environment variables are set:
   - LINEAR_API_KEY
   - DISCORD_BRIDGE_TOKEN
   - Other required service tokens

## Command to Execute

```bash
cd ~/5dlabs/cto && \
lobster run intake/workflows/pipeline.lobster.yaml \
  --var prd_content="$(cat ~/sigma-1/prd.md)" \
  --var project_name="sigma-1" \
  --var repository_url="https://github.com/5dlabs/sigma-1" \
  --var deliberate="true" \
  --var include_codebase="true"
```

## Alternative with Bridge Skip (for local testing)

If you want to skip Linear/Discord bridge requirements for local testing:

```bash
cd ~/5dlabs/cto && \
INTAKE_PREFLIGHT_BRIDGES_SKIP=true \
lobster run intake/workflows/pipeline.lobster.yaml \
  --var prd_content="$(cat ~/sigma-1/prd.md)" \
  --var project_name="sigma-1" \
  --var repository_url="https://github.com/5dlabs/sigma-1" \
  --var deliberate="true" \
  --var include_codebase="true"
```

## Expected Output

The pipeline will:
1. Analyze the existing codebase (if include_codebase=true)
2. Run the Optimist/Pessimist deliberation phase
3. Generate a structured task graph for implementation
4. Create Linear tickets for each task
5. Set up Discord communication channels
6. Generate implementation artifacts and documentation

## Post-Execution Steps

After successful pipeline execution:
1. Review generated Linear project and tasks
2. Verify Discord channel creation and integration
3. Check generated implementation artifacts
4. Begin assigning tasks to team members
5. Start implementation phase according to the 6-week plan