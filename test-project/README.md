# Minimal Test Project

A lightweight test project for validating the Play workflow without incurring costs.

## Purpose

This project provides a minimal set of 4 tasks that:
- Exercise the full Play workflow (task discovery, execution, completion)
- Demonstrate task dependencies and sequencing
- Create observable outputs (files) for validation
- Complete quickly (< 30 seconds per task)
- **Cost nothing** - no API calls, no cloud resources, no external services

## Tasks

1. **Task 1: Initialize** - Creates a hello.txt file
2. **Task 2: Process** - Transforms the file (uppercase)
3. **Task 3: Report** - Generates a markdown report
4. **Task 4: Finalize** - Creates final summary and completion marker

## Usage

### Option 1: Test with Play Project Workflow
```bash
# Submit the full project workflow
kubectl apply -f test-project-workflow.yaml
```

### Option 2: Test Individual Tasks
```bash
# Run just task 1
./test-single-task.sh 1

# Run tasks 1-3
./test-range.sh 1 3
```

## Expected Outputs

After successful execution, you should see:
- `hello.txt` - Created by Task 1
- `processed.txt` - Created by Task 2  
- `report.md` - Created by Task 3
- `final-summary.json` - Created by Task 4
- `.completed` - Marker file indicating project completion

## Validation

Check that:
1. Tasks execute in order (1 → 2 → 3 → 4)
2. Each task completes successfully
3. Files are created with expected content
4. No external API calls are made
5. Total execution time is < 5 minutes

## Benefits

- **Zero cost** - No cloud resources or API calls
- **Fast execution** - Each task < 30 seconds
- **Observable** - Creates files you can inspect
- **Repeatable** - Can run multiple times safely
- **Complete** - Exercises all workflow features
