# Task 7: Build Desktop Client (Spark - Electron)

**Agent**: spark | **Language**: typescript

## Role

You are a Desktop Engineer specializing in Electron implementing Task 7.

## Goal

Create the desktop system tray application using Electron 28+ with React for native desktop notifications, quick actions, and cross-platform support (Windows, macOS, Linux).

## Requirements

1. Initialize Electron project with React and TailwindCSS
2. Set up system tray with notification count badge
3. Implement native desktop notification system
4. Build main window with full notification feed
5. Create mini popup window for quick view
6. Add tray context menu with recent notifications
7. Implement keyboard shortcuts for common actions
8. Add settings window for preferences
9. Configure auto-start on system boot
10. Package for Windows, macOS, and Linux distribution

## Acceptance Criteria

Application runs on all target platforms, system tray icon appears with correct badge count, native notifications display properly, main and mini windows function correctly, keyboard shortcuts work, tray menu shows recent notifications, and auto-start configuration persists.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-7): Build Desktop Client (Spark - Electron)`

## Decision Points

### d13: How should the mini popup window be triggered?
**Category**: ux-behavior | **Constraint**: soft

Options:
1. hover-tray-icon
2. click-tray-icon
3. keyboard-shortcut
4. configurable

### d14: Should desktop client store authentication tokens securely in system keychain?
**Category**: security | **Constraint**: hard | ⚠️ **Requires Approval**

Options:
1. system-keychain
2. encrypted-local-storage
3. session-only


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-2, task-3, task-4
