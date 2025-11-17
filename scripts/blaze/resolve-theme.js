#!/usr/bin/env node

/**
 * 4-Layer Theme Resolution for Blaze Frontend Agent
 * 
 * Priority order (highest to lowest):
 * 1. Task-level overrides (from GitHub issue description)
 * 2. Repository design system (.blaze/design-system.json)
 * 3. Organization defaults (from Helm values / cto-config.json)
 * 4. Blaze built-in defaults
 * 
 * Usage:
 *   node resolve-theme.js [task-description] [repo-path]
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const taskDescription = process.argv[2] || '';
const repoPath = process.argv[3] || process.cwd();

// Blaze built-in default theme
const BUILT_IN_THEME = {
  name: 'Blaze Default Theme',
  version: '1.0',
  colors: {
    primary: { light: '#3B82F6', dark: '#60A5FA', foreground: '#FFFFFF' },
    secondary: { light: '#64748B', dark: '#94A3B8', foreground: '#FFFFFF' },
    accent: { light: '#10B981', dark: '#34D399', foreground: '#FFFFFF' },
    background: { light: '#FFFFFF', dark: '#0F172A' },
    foreground: { light: '#0F172A', dark: '#F8FAFC' },
    muted: { light: '#F1F5F9', dark: '#1E293B' },
    border: { light: '#E2E8F0', dark: '#334155' }
  },
  typography: {
    fontFamily: {
      sans: 'Inter, system-ui, -apple-system, sans-serif',
      heading: 'Inter, system-ui, -apple-system, sans-serif',
      mono: 'JetBrains Mono, Menlo, monospace'
    },
    fontSize: {
      xs: '0.75rem',
      sm: '0.875rem',
      base: '1rem',
      lg: '1.125rem',
      xl: '1.25rem',
      '2xl': '1.5rem',
      '3xl': '1.875rem',
      '4xl': '2.25rem',
      '5xl': '3rem'
    }
  },
  spacing: {
    scale: [0, 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96, 128],
    containerMaxWidth: '1280px',
    containerPadding: '1rem'
  },
  borderRadius: {
    none: '0',
    sm: '0.25rem',
    default: '0.5rem',
    lg: '0.75rem',
    xl: '1rem',
    full: '9999px'
  }
};

/**
 * Load repository design system if exists
 */
function loadRepoTheme(repoPath) {
  const designSystemPath = path.join(repoPath, '.blaze', 'design-system.json');
  
  if (fs.existsSync(designSystemPath)) {
    try {
      const content = fs.readFileSync(designSystemPath, 'utf8');
      const theme = JSON.parse(content);
      console.log('📁 Loaded repository design system:', theme.name || 'Untitled');
      return theme;
    } catch (error) {
      console.warn('⚠️  Failed to load repository design system:', error.message);
      return null;
    }
  }
  
  console.log('ℹ️  No repository design system found (.blaze/design-system.json)');
  return null;
}

/**
 * Parse task description for design preferences
 */
function parseTaskDesignPreferences(description) {
  const overrides = {};
  
  // Parse color preferences
  const colorMatch = description.match(/(?:color|colors?):\s*([^\n]+)/i);
  if (colorMatch) {
    const colorText = colorMatch[1];
    // Extract hex colors
    const hexMatches = colorText.match(/#[0-9A-Fa-f]{6}/g);
    if (hexMatches && hexMatches.length > 0) {
      overrides.colors = {
        primary: { light: hexMatches[0] }
      };
      console.log('🎨 Task color override:', hexMatches[0]);
    }
  }
  
  // Parse font preferences
  const fontMatch = description.match(/(?:font|typography):\s*([^\n]+)/i);
  if (fontMatch) {
    overrides.typography = {
      fontFamily: { sans: fontMatch[1].trim() }
    };
    console.log('📝 Task font override:', fontMatch[1].trim());
  }
  
  return Object.keys(overrides).length > 0 ? overrides : null;
}

/**
 * Deep merge themes (override takes precedence)
 */
function mergeTheme(base, override) {
  if (!override) return base;
  
  const merged = JSON.parse(JSON.stringify(base)); // Deep clone
  
  for (const key in override) {
    if (typeof override[key] === 'object' && !Array.isArray(override[key])) {
      merged[key] = mergeTheme(merged[key] || {}, override[key]);
    } else {
      merged[key] = override[key];
    }
  }
  
  return merged;
}

/**
 * Resolve final theme from all layers
 */
function resolveTheme(taskDescription, repoPath) {
  console.log('🔍 Resolving theme from 4 layers...');
  console.log('');
  
  // Layer 1: Built-in default
  let theme = BUILT_IN_THEME;
  console.log('1️⃣  Built-in default theme loaded');
  
  // Layer 2: Organization defaults (TODO: load from config)
  console.log('2️⃣  Organization defaults: Not configured (using built-in)');
  
  // Layer 3: Repository design system
  const repoTheme = loadRepoTheme(repoPath);
  if (repoTheme) {
    theme = mergeTheme(theme, repoTheme);
    console.log('3️⃣  Repository theme applied');
  } else {
    console.log('3️⃣  No repository theme (using defaults)');
  }
  
  // Layer 4: Task-level overrides
  const taskOverrides = parseTaskDesignPreferences(taskDescription);
  if (taskOverrides) {
    theme = mergeTheme(theme, taskOverrides);
    console.log('4️⃣  Task overrides applied');
  } else {
    console.log('4️⃣  No task overrides (using repository/default)');
  }
  
  console.log('');
  console.log('✅ Final theme resolved');
  
  return theme;
}

// Main execution
const resolvedTheme = resolveTheme(taskDescription, repoPath);

// Output theme for consumption by other scripts
console.log('');
console.log('📄 Resolved Theme:');
console.log(JSON.stringify(resolvedTheme, null, 2));

// Save to file for container script using a securely created temp path, then move to stable location
const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'resolved-theme-'));
const tempFile = path.join(tempDir, 'theme.json');
const outputPath = path.join(os.tmpdir(), 'resolved-theme.json');
fs.writeFileSync(tempFile, JSON.stringify(resolvedTheme, null, 2), { mode: 0o600 });
fs.renameSync(tempFile, outputPath);
fs.rmSync(tempDir, { recursive: true, force: true });
console.log('');
console.log(`💾 Theme saved to: ${outputPath}`);

