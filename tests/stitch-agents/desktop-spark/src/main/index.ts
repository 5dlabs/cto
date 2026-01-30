// Code smells for Spark (Electron) to find:
// - Security issues (nodeIntegration, contextIsolation)
// - Sync IPC calls
// - Remote module usage
// - No input validation

import { app, BrowserWindow, ipcMain } from 'electron';
import * as path from 'path';
import * as fs from 'fs';

let mainWindow: BrowserWindow | null = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      // SECURITY ISSUES:
      nodeIntegration: true,        // Bad - allows Node in renderer
      contextIsolation: false,      // Bad - no context isolation
      webSecurity: false,           // Bad - disables same-origin policy
    }
  });

  mainWindow.loadFile('index.html');
}

// Synchronous IPC - blocks main process
ipcMain.on('sync-read-file', (event, filePath) => {
  // No input validation - path traversal vulnerability!
  const content = fs.readFileSync(filePath, 'utf8');
  event.returnValue = content;
});

// Executing arbitrary commands - dangerous!
ipcMain.handle('execute-command', async (event, cmd: any) => {
  const { exec } = require('child_process');
  return new Promise((resolve) => {
    exec(cmd, (error: any, stdout: any) => {
      resolve(stdout);
    });
  });
});

app.whenReady().then(createWindow);
