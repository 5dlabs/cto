# Minimal Task 3 Prompt — Create PR

Goal: Make a trivial change and open a Pull Request labeled with this task.

Do exactly the following:
- Create a file `task/HELLO-3.md` with the content: `Hello from Task 3`
- Stage and commit the change on the current branch with message: `chore(task-3): add HELLO-3.md`
- Open a PR from this branch to `main` with title: `test(task-3): minimal change for validation`
- Add the label `task-3` to the PR
- Do not install dependencies or run external services; keep it zero-cost

Success criteria:
- A PR exists targeting `main`
- PR has label `task-3`
- The file `task/HELLO-3.md` is present in the PR diff
