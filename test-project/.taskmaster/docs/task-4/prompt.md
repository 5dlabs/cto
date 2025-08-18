# Minimal Task 4 Prompt — Create PR

Goal: Make a trivial change and open a Pull Request labeled with this task.

Do exactly the following:
- Create a file `task/HELLO-4.md` with the content: `Hello from Task 4`
- Stage and commit the change on the current branch with message: `chore(task-4): add HELLO-4.md`
- Open a PR from this branch to `main` with title: `test(task-4): minimal change for validation`
- Add the label `task-4` to the PR
- Do not install dependencies or run external services; keep it zero-cost

Success criteria:
- A PR exists targeting `main`
- PR has label `task-4`
- The file `task/HELLO-4.md` is present in the PR diff
