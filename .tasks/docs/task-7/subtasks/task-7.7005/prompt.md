Implement subtask 7005: Add research memo count to summary header

## Objective
Update the pipeline dashboard summary header to display a count of how many tasks have research memos out of the total, e.g., '3 of 5 tasks have research memos'.

## Steps
1. In the summary header component/section of the pipeline dashboard page, compute the count: `const memoCount = tasks.filter(t => t.research_memo != null).length`. 2. Display: `{memoCount} of {tasks.length} tasks have research memos`. 3. Place this line alongside existing summary statistics (e.g., task counts by status). 4. If memoCount is 0, display '0 of N tasks have research memos' (still informative).

## Validation
Component test: render summary header with an array of 5 tasks where 3 have non-null research_memo; verify text '3 of 5 tasks have research memos' is displayed. Test with 0 memos; verify '0 of 5 tasks have research memos'.