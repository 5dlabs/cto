# Acceptance Criteria



- Enumerates task directories `task-*` in ascending numeric order


- Submits `play-workflow-template` once per task with `-p task-id=<n>`


- Runs one task at a time (never more than one CodeRun Job concurrently)


- Detects task completion (end of Play workflow) before advancing


- Supports resuming from a given starting task (`start-from`)


- Logs progress per task and overall summary at the end


- Verified run across at least three tasks (1→3) in sequence
