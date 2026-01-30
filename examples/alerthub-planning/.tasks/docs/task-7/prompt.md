# Implementation Prompt for Task 7

## Context
You are implementing "Desktop System Tray Client (Spark - Electron)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Create desktop application using Electron 28+ that runs in system tray and displays native notifications with quick actions.

## Implementation Details
Build Electron app with system tray integration, native desktop notifications, main window with notification feed, mini popup window, settings window, keyboard shortcuts, and auto-start functionality. Support Windows, macOS, and Linux.

## Dependencies
This task depends on: task-2, task-4. Ensure those are complete before starting.

## Testing Requirements
App builds for all platforms, system tray icon appears with correct badge count, native notifications display, keyboard shortcuts work, auto-start functions correctly, and all windows open/close properly

## Decision Points to Address

The following decisions need to be made during implementation:

### d13: System tray notification display strategy
**Category**: ux-behavior | **Constraint**: soft

Options:
1. show last 5 notifications in tray menu
2. show unread count only
3. show preview of latest notification

Document your choice and rationale in the implementation.

### d14: WebSocket connection management for desktop app
**Category**: performance | **Constraint**: open

Options:
1. persistent connection with auto-reconnect
2. connect only when app is active
3. configurable connection strategy

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
