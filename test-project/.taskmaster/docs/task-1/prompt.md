# Minimal Task 1 Prompt — Create PR

Goal: Make a trivial change and open a Pull Request labeled with this task.

Do exactly the following:
- Create a file `task/HELLO-1.md` with the content: `Hello from Task 1`
- Stage and commit the change on the current branch with message: `chore(task-1): add HELLO-1.md`
- Open a PR from this branch to `main` with title: `test(task-1): minimal change for validation`
- Add the label `task-1` to the PR
- Do not install dependencies or run external services; keep it zero-cost

Success criteria:
- A PR exists targeting `main`
- PR has label `task-1`
- The file `task/HELLO-1.md` is present in the PR diff

