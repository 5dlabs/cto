# Task 7: Desktop System Tray Client (Spark - Electron)

**Agent**: spark | **Language**: typescript

## Role

You are a Desktop Engineer specializing in Electron implementing Task 7.

## Goal

Create desktop application using Electron 28+ that runs in system tray and displays native notifications with quick actions.

## Requirements

Build Electron app with system tray integration, native desktop notifications, main window with notification feed, mini popup window, settings window, keyboard shortcuts, and auto-start functionality. Support Windows, macOS, and Linux.

## Acceptance Criteria

App builds for all platforms, system tray icon appears with correct badge count, native notifications display, keyboard shortcuts work, auto-start functions correctly, and all windows open/close properly

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-7): Desktop System Tray Client (Spark - Electron)`

## Decision Points

### d13: System tray notification display strategy
**Category**: ux-behavior | **Constraint**: soft

Options:
1. show last 5 notifications in tray menu
2. show unread count only
3. show preview of latest notification

### d14: WebSocket connection management for desktop app
**Category**: performance | **Constraint**: open

Options:
1. persistent connection with auto-reconnect
2. connect only when app is active
3. configurable connection strategy


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-2, task-4
