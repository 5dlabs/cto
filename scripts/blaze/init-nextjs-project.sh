#!/bin/bash
set -euo pipefail

# ============================================================================
# Blaze - Next.js + shadcn/ui Project Initialization Script
# ============================================================================
# This script initializes a production-ready Next.js project with:
# - TypeScript 5 (strict mode)
# - Tailwind CSS 4
# - shadcn/ui component library
# - React 19 + Next.js 15 (App Router)
# - Proper project structure
# ============================================================================

echo "🎨 Blaze: Initializing Next.js + shadcn/ui project..."

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

# Check if Next.js project already exists
if [ -f "package.json" ] && grep -q "next" package.json 2>/dev/null; then
    echo "✅ Next.js project already exists, skipping initialization"
    exit 0
fi

# Ensure pnpm is available
if ! command -v pnpm &> /dev/null; then
    echo "📦 Installing pnpm..."
    npm install -g pnpm
fi

echo "🚀 Creating Next.js 15 project with TypeScript..."

# Create Next.js project with recommended settings
pnpm create next-app@latest . \
    --typescript \
    --tailwind \
    --app \
    --no-src-dir \
    --import-alias "@/*" \
    --use-pnpm \
    --skip-install

echo "📦 Installing dependencies..."
pnpm install

echo "🎨 Initializing shadcn/ui..."

# Create shadcn/ui components directory
mkdir -p components/ui

# Initialize shadcn/ui with default configuration
cat > components.json <<EOF
{
  "\$schema": "https://ui.shadcn.com/schema.json",
  "style": "default",
  "rsc": true,
  "tsx": true,
  "tailwind": {
    "config": "tailwind.config.ts",
    "css": "app/globals.css",
    "baseColor": "slate",
    "cssVariables": true,
    "prefix": ""
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "hooks": "@/hooks"
  }
}
EOF

# Install shadcn/ui core dependencies
echo "📦 Installing shadcn/ui dependencies..."
pnpm add class-variance-authority clsx tailwind-merge
pnpm add -D @types/node

# Create lib/utils.ts for cn() helper
mkdir -p lib
cat > lib/utils.ts <<'EOF'
import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
EOF

# Install commonly used shadcn/ui components
echo "🧩 Installing common shadcn/ui components..."
pnpm dlx shadcn@latest add button card dialog input label form --yes --overwrite

# Install additional dependencies for forms and validation
pnpm add react-hook-form zod @hookform/resolvers

# Update tailwind.config.ts for shadcn/ui
echo "⚙️  Configuring Tailwind CSS..."
cat > tailwind.config.ts <<'EOF'
import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class"],
  content: [
    "./pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};

export default config;
EOF

# Install tailwindcss-animate
pnpm add tailwindcss-animate

# Update app/globals.css with shadcn/ui theme variables
cat > app/globals.css <<'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;
    --primary: 221.2 83.2% 53.3%;
    --primary-foreground: 210 40% 98%;
    --secondary: 210 40% 96.1%;
    --secondary-foreground: 222.2 47.4% 11.2%;
    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;
    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 221.2 83.2% 53.3%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --card: 222.2 84% 4.9%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 84% 4.9%;
    --popover-foreground: 210 40% 98%;
    --primary: 217.2 91.2% 59.8%;
    --primary-foreground: 222.2 47.4% 11.2%;
    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;
    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;
    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;
    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 224.3 76.3% 48%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
  }
}
EOF

# Update TypeScript config for strict mode
echo "⚙️  Configuring TypeScript (strict mode)..."
cat > tsconfig.json <<'EOF'
{
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [
      {
        "name": "next"
      }
    ],
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
EOF

# Create a sample component to demonstrate setup
echo "🧩 Creating sample component..."
mkdir -p components
cat > components/hero.tsx <<'EOF'
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

export function Hero() {
  return (
    <div className="container mx-auto px-4 py-16">
      <Card className="p-8">
        <h1 className="text-4xl font-bold mb-4">
          Welcome to Your Next.js App
        </h1>
        <p className="text-muted-foreground mb-6">
          Built with Next.js 15, React 19, TypeScript, and shadcn/ui
        </p>
        <Button size="lg">Get Started</Button>
      </Card>
    </div>
  );
}
EOF

# Update app/page.tsx with sample usage
cat > app/page.tsx <<'EOF'
import { Hero } from "@/components/hero";

export default function Home() {
  return (
    <main className="min-h-screen">
      <Hero />
    </main>
  );
}
EOF

# Add useful scripts to package.json
echo "📝 Adding development scripts..."
if [ -f "package.json" ]; then
    # Use node to update package.json
    node -e "
    const fs = require('fs');
    const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
    pkg.scripts = {
      ...pkg.scripts,
      'type-check': 'tsc --noEmit',
      'format': 'prettier --write .',
      'format:check': 'prettier --check .'
    };
    fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
fi

# Install Prettier for code formatting
echo "📦 Installing Prettier..."
pnpm add -D prettier prettier-plugin-tailwindcss

# Create Prettier config
cat > .prettierrc <<'EOF'
{
  "semi": true,
  "trailingComma": "es5",
  "singleQuote": false,
  "printWidth": 100,
  "tabWidth": 2,
  "plugins": ["prettier-plugin-tailwindcss"]
}
EOF

# Create .prettierignore
cat > .prettierignore <<'EOF'
.next
node_modules
.git
.blaze
*.json
EOF

echo ""
echo "✅ Next.js + shadcn/ui project initialized successfully!"
echo ""
echo "📦 Installed components:"
echo "   - button, card, dialog, input, label, form"
echo ""
echo "🛠️  Available commands:"
echo "   pnpm dev          # Start development server"
echo "   pnpm build        # Build for production"
echo "   pnpm start        # Start production server"
echo "   pnpm lint         # Run ESLint"
echo "   pnpm type-check   # Run TypeScript checks"
echo "   pnpm format       # Format code with Prettier"
echo ""
echo "📚 Next steps:"
echo "   1. Add more shadcn/ui components: pnpm dlx shadchn add <component>"
echo "   2. Customize theme in tailwind.config.ts"
echo "   3. Start building components in components/"
echo ""

