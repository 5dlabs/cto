# Task 13: Build and Distribute Desktop Client (Spark - Electron)

**Agent**: spark | **Language**: typescript

## Role

You are a Senior Desktop Engineer with expertise in Electron and native integrations implementing Task 13.

## Goal

Build production installers for Windows, macOS, and Linux and setup auto-update

## Requirements

1. Configure electron-builder in package.json:
   "build": {
     "appId": "com.alerthub.desktop",
     "productName": "AlertHub",
     "directories": { "output": "dist" },
     "files": ["build/**/*", "node_modules/**/*", "package.json"],
     "mac": { "category": "public.app-category.productivity", "target": ["dmg", "zip"] },
     "win": { "target": ["nsis", "portable"] },
     "linux": { "target": ["AppImage", "deb"] }
   }

2. Setup code signing:
   - macOS: Obtain Apple Developer certificate, configure CSC_LINK and CSC_KEY_PASSWORD env vars
   - Windows: Obtain code signing certificate, configure CSC_LINK and CSC_KEY_PASSWORD env vars

3. Build for all platforms:
   - macOS: npm run dist -- --mac
   - Windows: npm run dist -- --win
   - Linux: npm run dist -- --linux

4. Setup auto-update with electron-updater:
   - Configure update server URL in main process
   - Add update check on app startup
   - Implement update download and install flow
   - Test update with mock server

5. Create distribution packages:
   - macOS: DMG installer with drag-and-drop
   - Windows: NSIS installer with auto-update support
   - Linux: AppImage and Deb package

6. Setup GitHub Releases for distribution:
   - Create GitHub release with version tag
   - Upload installers to release assets
   - Configure electron-updater to check GitHub releases

7. Create download page:
   - Add download links to web console
   - Detect user OS and suggest appropriate installer
   - Provide checksums for verification

8. Test installers:
   - Install on clean Windows 10/11 VM
   - Install on macOS 12+ (Intel and Apple Silicon)
   - Install on Ubuntu 22.04 LTS
   - Verify app launches and functions correctly
   - Test auto-update flow

## Acceptance Criteria

1. Test installers on all target platforms
2. Verify code signing (no security warnings)
3. Test auto-update from previous version
4. Test system tray integration
5. Test global shortcuts
6. Test native notifications
7. Verify WebSocket connection
8. Test multi-window behavior
9. Verify auto-launch on boot
10. Check app size (target: <100MB)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-13): Build and Distribute Desktop Client (Spark - Electron)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 7, 8, 9, 10
