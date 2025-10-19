#!/usr/bin/env node
/**
 * Blaze Theme Resolution
 * Resolves themes from multiple sources in priority order:
 * 1. Task description overrides (highest)
 * 2. Repository design system (.blaze/design-system.json)
 * 3. Organization defaults (from env)
 * 4. Built-in defaults (lowest)
 */

const fs = require('fs');
const path = require('path');

// Built-in default theme
const BUILTIN_THEME = {
  name: "Blaze Default Theme",
  version: "1.0",
  colors: {
    primary: { light: "#3B82F6", dark: "#60A5FA", foreground: "#FFFFFF" },
    secondary: { light: "#64748B", dark: "#94A3B8", foreground: "#FFFFFF" },
    accent: { light: "#10B981", dark: "#34D399", foreground: "#FFFFFF" },
    background: { light: "#FFFFFF", dark: "#0F172A" },
    foreground: { light: "#0F172A", dark: "#F8FAFC" },
    muted: { light: "#F1F5F9", dark: "#1E293B" },
    border: { light: "#E2E8F0", dark: "#334155" }
  },
  typography: {
    fontFamily: {
      sans: "Inter, system-ui, -apple-system, sans-serif",
      heading: "Inter, system-ui, -apple-system, sans-serif",
      mono: "JetBrains Mono, Menlo, monospace"
    },
    fontSize: {
      xs: "0.75rem", sm: "0.875rem", base: "1rem",
      lg: "1.125rem", xl: "1.25rem", "2xl": "1.5rem",
      "3xl": "1.875rem", "4xl": "2.25rem", "5xl": "3rem"
    },
    fontWeight: { normal: 400, medium: 500, semibold: 600, bold: 700 },
    lineHeight: { tight: 1.25, normal: 1.5, relaxed: 1.75 }
  },
  spacing: {
    scale: [0, 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96, 128],
    containerMaxWidth: "1280px",
    containerPadding: "1rem"
  },
  borderRadius: {
    none: "0", sm: "0.25rem", default: "0.5rem", md: "0.5rem",
    lg: "0.75rem", xl: "1rem", "2xl": "1.5rem", full: "9999px"
  },
  shadows: {
    sm: "0 1px 2px 0 rgb(0 0 0 / 0.05)",
    default: "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
    md: "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
    lg: "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
    xl: "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)"
  }
};

function deepMerge(target, source) {
  const output = { ...target };
  if (isObject(target) && isObject(source)) {
    Object.keys(source).forEach(key => {
      if (isObject(source[key])) {
        if (!(key in target)) {
          Object.assign(output, { [key]: source[key] });
        } else {
          output[key] = deepMerge(target[key], source[key]);
        }
      } else {
        Object.assign(output, { [key]: source[key] });
      }
    });
  }
  return output;
}

function isObject(item) {
  return item && typeof item === 'object' && !Array.isArray(item);
}

function loadRepoTheme(repoPath) {
  const themeFile = path.join(repoPath, '.blaze', 'design-system.json');
  if (fs.existsSync(themeFile)) {
    try {
      const theme = JSON.parse(fs.readFileSync(themeFile, 'utf8'));
      console.error('📘 Loaded repository design system from .blaze/design-system.json');
      return theme;
    } catch (err) {
      console.error('⚠️  Failed to parse .blaze/design-system.json:', err.message);
    }
  }
  return null;
}

function parseTaskDesignPreferences(taskDescription) {
  // Simple extraction of design preferences from markdown
  const designSection = taskDescription.match(/##\s*Design.*?\n([\s\S]*?)(?=##|$)/i);
  if (!designSection) return null;

  const prefs = {};
  const content = designSection[1];

  // Extract colors
  const primaryColor = content.match(/primary[:\s]+([#\w]+)/i);
  if (primaryColor) {
    prefs.colors = prefs.colors || {};
    prefs.colors.primary = { light: primaryColor[1] };
  }

  // Extract font family
  const fontFamily = content.match(/font[:\s]+([^\n]+)/i);
  if (fontFamily) {
    prefs.typography = prefs.typography || {};
    prefs.typography.fontFamily = {
      sans: fontFamily[1].trim()
    };
  }

  return Object.keys(prefs).length > 0 ? prefs : null;
}

function resolveTheme(taskDescription, repoPath, orgDefaults = null) {
  let theme = { ...BUILTIN_THEME };

  // Apply organization defaults (if configured via env)
  if (orgDefaults) {
    console.error('🏢 Applying organization defaults');
    theme = deepMerge(theme, orgDefaults);
  }

  // Apply repository design system
  const repoTheme = loadRepoTheme(repoPath);
  if (repoTheme) {
    theme = deepMerge(theme, repoTheme);
  }

  // Apply task-level overrides
  const taskOverrides = parseTaskDesignPreferences(taskDescription);
  if (taskOverrides) {
    console.error('📝 Applying task-level design overrides');
    theme = deepMerge(theme, taskOverrides);
  }

  return theme;
}

// CLI usage
if (require.main === module) {
  const taskDescription = process.argv[2] || process.env.TASK_DESCRIPTION || '';
  const repoPath = process.argv[3] || process.env.REPO_PATH || process.cwd();

  // Optional: Load org defaults from env (JSON string)
  let orgDefaults = null;
  if (process.env.BLAZE_ORG_DEFAULTS) {
    try {
      orgDefaults = JSON.parse(process.env.BLAZE_ORG_DEFAULTS);
    } catch (err) {
      console.error('⚠️  Failed to parse BLAZE_ORG_DEFAULTS:', err.message);
    }
  }

  const resolvedTheme = resolveTheme(taskDescription, repoPath, orgDefaults);

  // Output as JSON
  console.log(JSON.stringify(resolvedTheme, null, 2));
}

module.exports = { resolveTheme, BUILTIN_THEME };

