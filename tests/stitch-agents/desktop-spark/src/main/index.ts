/**
 * Stitch test fixture for Spark (Electron desktop agent)
 * 
 * This file contains intentional issues for testing remediation:
 * - Security issues (nodeIntegration)
 * - Missing error handling
 * - Type errors
 */

import { app, BrowserWindow, ipcMain } from 'electron'
import path from 'path'

// TODO: Intentional issue - unused variable
const unusedConfig = {
  autoUpdate: true,
  telemetry: false,
}

function createWindow() {
  // TODO: Intentional issue - security risk with nodeIntegration
  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: true, // Security issue!
      contextIsolation: false, // Security issue!
      preload: path.join(__dirname, 'preload.js'),
    },
  })

  // TODO: Intentional issue - no error handling
  mainWindow.loadFile('index.html')
}

// TODO: Intentional issue - any type
ipcMain.handle('get-users', async (event: any) => {
  // Simulated user fetch
  return [
    { id: 1, name: 'Alice', email: 'alice@example.com' },
    { id: 2, name: 'Bob', email: 'bob@example.com' },
  ]
})

app.whenReady().then(() => {
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
