# Minimal Task 2 Prompt — Create PR

Goal: Make a trivial change and open a Pull Request labeled with this task.

Do exactly the following:
- Create a file `task/HELLO-2.md` with the content: `Hello from Task 2`
- Stage and commit the change on the current branch with message: `chore(task-2): add HELLO-2.md`
- Open a PR from this branch to `main` with title: `test(task-2): minimal change for validation`
- Add the label `task-2` to the PR
- Do not install dependencies or run external services; keep it zero-cost

Success criteria:
- A PR exists targeting `main`
- PR has label `task-2`
- The file `task/HELLO-2.md` is present in the PR diff
