# Task 7: Implement Desktop Client (Spark - Electron)

**Agent**: spark | **Language**: typescript

## Role

You are a Senior Desktop Engineer with expertise in Electron and native integrations implementing Task 7.

## Goal

Build the cross-platform desktop application with system tray integration and native notifications

## Requirements

1. Initialize Electron project:
   npm init -y
   npm install electron electron-builder electron-updater react react-dom
   npm install -D @types/react @types/react-dom typescript webpack webpack-cli

2. Setup project structure:
   src/main/main.ts - Electron main process
   src/renderer/App.tsx - React app
   src/preload/preload.ts - Preload script (IPC bridge)
   public/ - Static assets (icons, HTML)

3. Implement main process (src/main/main.ts):
   - Create BrowserWindow with preload script
   - Setup system tray with icon and menu
   - Register global shortcuts (Cmd/Ctrl+Shift+A to show/hide)
   - Handle app lifecycle (ready, window-all-closed, activate)
   - Setup auto-updater with electron-updater
   - Configure auto-launch on system boot

4. Implement system tray:
   - Create tray icon with unread count badge
   - Build context menu:
     - Show/Hide main window
     - Recent notifications (last 5)
     - Mute for 1 hour / until tomorrow
     - Preferences
     - Quit
   - Update badge count on notification received

5. Implement IPC communication:
   - main -> renderer: notification-received, settings-updated
   - renderer -> main: show-notification, update-settings, quit-app
   - Use contextBridge in preload script for secure IPC

6. Build renderer React app:
   - MainWindow: Full notification feed with filters
   - MiniWindow: Quick view popup (200x400px)
   - SettingsWindow: Preferences and account settings

7. Implement MainWindow component:
   - Notification list with virtual scrolling
   - Filters (status, channel, date range)
   - Search bar
   - Action buttons (mark as read, delete, refresh)
   - WebSocket connection for real-time updates

8. Implement MiniWindow component:
   - Compact notification cards (last 10)
   - Quick actions (dismiss, view details)
   - Click to open MainWindow with selected notification

9. Implement SettingsWindow component:
   - Notification preferences (enable/disable channels)
   - Desktop notification settings (sound, position)
   - Startup options (launch on boot, start minimized)
   - Account info and logout

10. Implement native desktop notifications:
    - Use Electron Notification API
    - Show notification with title, body, icon
    - Handle notification click (focus window, navigate to detail)
    - Respect user preferences (quiet hours, mute)

11. Configure electron-builder:
    - Set app name, product name, app ID
    - Configure build targets (Windows NSIS, macOS DMG, Linux AppImage)
    - Set icon paths for each platform
    - Configure auto-update settings

12. Create Dockerfile for build environment:
   FROM electronuserland/builder:wine
   WORKDIR /app
   COPY package.json package-lock.json ./
   RUN npm ci
   COPY . .
   RUN npm run build
   CMD ["npm", "run", "dist"]

## Acceptance Criteria

1. Unit tests for IPC handlers
2. Test system tray menu actions
3. Test global shortcuts
4. Test native notifications
5. Test auto-updater with mock updates
6. Test WebSocket connection and real-time updates
7. Manual testing on Windows, macOS, Linux
8. Test app lifecycle (minimize, restore, quit)
9. Verify auto-launch on boot
10. Test multi-window behavior (main, mini, settings)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-7): Implement Desktop Client (Spark - Electron)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 2, 3, 4
