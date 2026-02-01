/**
 * Electron main process entry point.
 * Test fixture for Spark agent detection.
 */

import { app, BrowserWindow, ipcMain } from 'electron';
import * as path from 'path';

let mainWindow: BrowserWindow | null = null;

// Subtle: any type usage
function createWindow(options: any) {
  mainWindow = new BrowserWindow({
    width: options.width || 800,
    height: options.height || 600,
    webPreferences: {
      nodeIntegration: true,  // Subtle: security concern
      contextIsolation: false, // Subtle: security concern
    },
  });

  mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
  
  // Subtle: not checking if window exists before access
  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

app.whenReady().then(() => {
  createWindow({});
  
  // Subtle: redundant condition
  if (process.platform !== 'darwin') {
    // Non-macOS behavior
  } else if (process.platform === 'darwin') {
    // macOS behavior - redundant check
    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length == 0) {  // Subtle: == instead of ===
        createWindow({});
      }
    });
  }
});

// Subtle: sync IPC handler in main process
ipcMain.on('sync-message', (event, arg) => {
  console.log('Received:', arg);  // Subtle: console.log
  event.returnValue = 'pong';
});

// Subtle: magic string
ipcMain.handle('get-app-path', async () => {
  return app.getPath('userData');
});

app.on('window-all-closed', () => {
  if (process.platform != 'darwin') {  // Subtle: != instead of !==
    app.quit();
  }
});
