# Subtask 46.2: Implement core global shortcuts manager

## Parent Task
Task 46

## Subagent Type
implementer

## Agent
code-implementer

## Parallelizable
Yes - can run concurrently

## Description
Create the global shortcuts manager module that handles registration, unregistration, and management of keyboard shortcuts using Electron's globalShortcut API.

## Dependencies
- Subtask 46.1

## Implementation Details
Implement a GlobalShortcutsManager class that wraps Electron's globalShortcut API, provides methods for registering/unregistering shortcuts, handles cleanup on app exit, and includes error handling for shortcut conflicts. Include configuration loading and user preference support.

## Test Strategy
Unit tests for shortcut registration/unregistration and conflict handling
