#!/bin/bash
set -euo pipefail

# ===========================================================================
# Spark - Expo + NativeWind Project Initialization Script
# ===========================================================================
# This script initializes a production-ready React Native/Expo project with:
# - TypeScript 5 (strict mode)
# - NativeWind (Tailwind for React Native)
# - Expo Router for navigation
# - React Native Safe Area Context
# - Proper project structure
# ===========================================================================

echo "📱 Spark: Initializing Expo + NativeWind project..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository. Please run this from a git repo."
    exit 1
fi

# Get project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
WORKSPACE_DIR="${1:-$PWD}"

cd "$WORKSPACE_DIR"

echo "📍 Working directory: $WORKSPACE_DIR"

# Check if Expo project already exists
if [ -f "app.json" ] || [ -f "app.config.js" ] || [ -f "app.config.ts" ]; then
    echo "✅ Expo project already exists, skipping initialization"
    exit 0
fi

# Check if package.json exists with expo
if [ -f "package.json" ] && grep -q "expo" package.json 2>/dev/null; then
    echo "✅ Expo project already exists (detected in package.json), skipping initialization"
    exit 0
fi

# Ensure npm is available
if ! command -v npm &> /dev/null; then
    echo "❌ Error: npm is not installed"
    exit 1
fi

echo "🚀 Creating Expo app with TypeScript..."

# Create Expo project with tabs template (includes TypeScript and navigation)
npx create-expo-app@latest . \
    --template tabs \
    --yes

echo "📦 Installing NativeWind and dependencies..."

# Install NativeWind
npx expo install nativewind tailwindcss

# Install react-native-reanimated (required for many animations)
npx expo install react-native-reanimated

# Install additional common dependencies
npx expo install expo-status-bar expo-constants expo-linking
npx expo install @react-native-async-storage/async-storage
npx expo install react-native-safe-area-context
npx expo install react-native-gesture-handler

echo "⚙️ Configuring Tailwind CSS..."

# Create tailwind.config.js
cat > tailwind.config.js <<'EOF'
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/**/*.{js,jsx,ts,tsx}",
    "./components/**/*.{js,jsx,ts,tsx}",
  ],
  presets: [require("nativewind/preset")],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#3b82f6",
          foreground: "#ffffff",
        },
        secondary: {
          DEFAULT: "#64748b",
          foreground: "#ffffff",
        },
        destructive: {
          DEFAULT: "#ef4444",
          foreground: "#ffffff",
        },
        muted: {
          DEFAULT: "#f1f5f9",
          foreground: "#64748b",
        },
        accent: {
          DEFAULT: "#f1f5f9",
          foreground: "#0f172a",
        },
        background: "#ffffff",
        foreground: "#0f172a",
        card: {
          DEFAULT: "#ffffff",
          foreground: "#0f172a",
        },
        border: "#e2e8f0",
        input: "#e2e8f0",
        ring: "#3b82f6",
      },
      borderRadius: {
        lg: "0.75rem",
        md: "0.5rem",
        sm: "0.25rem",
      },
    },
  },
  plugins: [],
};
EOF

echo "⚙️ Configuring Babel for NativeWind..."

# Update babel.config.js
cat > babel.config.js <<'EOF'
module.exports = function (api) {
  api.cache(true);
  return {
    presets: [
      ["babel-preset-expo", { jsxImportSource: "nativewind" }],
      "nativewind/babel",
    ],
  };
};
EOF

echo "⚙️ Creating global.css for NativeWind..."

# Create global.css
cat > global.css <<'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;
EOF

echo "📁 Creating component directories..."

# Create component directories
mkdir -p components/ui
mkdir -p hooks
mkdir -p lib
mkdir -p constants

echo "🧩 Creating base UI components..."

# Create Button component
cat > components/ui/button.tsx <<'EOF'
import { forwardRef } from 'react';
import { Pressable, Text, PressableProps, View } from 'react-native';

export interface ButtonProps extends Omit<PressableProps, 'children'> {
  title: string;
  variant?: 'default' | 'secondary' | 'destructive' | 'outline' | 'ghost';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  className?: string;
}

const variantStyles = {
  default: 'bg-primary',
  secondary: 'bg-secondary',
  destructive: 'bg-destructive',
  outline: 'border-2 border-input bg-transparent',
  ghost: 'bg-transparent',
};

const variantTextStyles = {
  default: 'text-primary-foreground',
  secondary: 'text-secondary-foreground',
  destructive: 'text-destructive-foreground',
  outline: 'text-foreground',
  ghost: 'text-foreground',
};

const sizeStyles = {
  default: 'px-6 py-3',
  sm: 'px-4 py-2',
  lg: 'px-8 py-4',
  icon: 'p-3',
};

export const Button = forwardRef<View, ButtonProps>(
  ({ title, variant = 'default', size = 'default', className = '', disabled, ...props }, ref) => {
    return (
      <Pressable
        ref={ref}
        className={`items-center justify-center rounded-lg active:opacity-80 ${variantStyles[variant]} ${sizeStyles[size]} ${disabled ? 'opacity-50' : ''} ${className}`}
        disabled={disabled}
        accessibilityRole="button"
        accessibilityLabel={title}
        accessibilityState={{ disabled }}
        {...props}
      >
        <Text className={`font-semibold ${variantTextStyles[variant]}`}>
          {title}
        </Text>
      </Pressable>
    );
  }
);

Button.displayName = 'Button';
EOF

# Create component index
cat > components/ui/index.ts <<'EOF'
export * from './button';
EOF

echo "📝 Creating lib utilities..."

# Create cn utility
cat > lib/utils.ts <<'EOF'
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
EOF

# Install clsx and tailwind-merge
npm install clsx tailwind-merge

echo ""
echo "✅ Expo + NativeWind project initialized successfully!"
echo ""
echo "🛠️ Available commands:"
echo "   npx expo start          # Start development server"
echo "   npx expo start --ios    # Start on iOS simulator"
echo "   npx expo start --android # Start on Android emulator"
echo "   npm run lint            # Run ESLint"
echo ""
