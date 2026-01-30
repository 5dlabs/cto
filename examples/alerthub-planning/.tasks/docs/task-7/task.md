# Task 7: Desktop System Tray Client (Spark - Electron)

## Status
pending

## Priority
medium

## Dependencies
task-2, task-4

## Description
Create desktop application using Electron 28+ that runs in system tray and displays native notifications with quick actions.

## Details
Build Electron app with system tray integration, native desktop notifications, main window with notification feed, mini popup window, settings window, keyboard shortcuts, and auto-start functionality. Support Windows, macOS, and Linux.

## Test Strategy
App builds for all platforms, system tray icon appears with correct badge count, native notifications display, keyboard shortcuts work, auto-start functions correctly, and all windows open/close properly

## Decision Points

### d13: System tray notification display strategy
- **Category**: ux-behavior
- **Constraint**: soft
- **Requires Approval**: No
- **Options**:
  - show last 5 notifications in tray menu
  - show unread count only
  - show preview of latest notification

### d14: WebSocket connection management for desktop app
- **Category**: performance
- **Constraint**: open
- **Requires Approval**: No
- **Options**:
  - persistent connection with auto-reconnect
  - connect only when app is active
  - configurable connection strategy

