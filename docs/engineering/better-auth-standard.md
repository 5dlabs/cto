# BetterAuth: Standard Authentication for CTO Platform

**Date:** 2025-11-24  
**Status:** Official Authentication Standard  
**Purpose:** Define BetterAuth as the default authentication solution for all CTO-built applications

---

## Executive Summary

BetterAuth is now the **official authentication standard** for all applications built by the CTO platform. This framework-agnostic, TypeScript-first authentication solution provides enterprise-grade security features out of the box, enabling our agents to rapidly implement secure, modern authentication.

---

## Why BetterAuth?

### Core Benefits

1. **Framework Agnostic**: Works with Next.js, Remix, Astro, SvelteKit, Vue, etc.
2. **Database Flexible**: Supports Prisma, Drizzle, Kysely, PostgreSQL, MySQL, SQLite
3. **TypeScript First**: Full type safety throughout the stack
4. **Plugin Architecture**: Extensible for custom requirements
5. **AI Integration**: Built-in MCP server for agent interaction

### Feature Set

| Category | Features |
|----------|----------|
| **Core Auth** | Email/Password, Magic Links, Username |
| **Social Login** | GitHub, Google, Discord, 40+ providers |
| **Security** | 2FA/MFA, Passkeys/WebAuthn, Device Management |
| **Enterprise** | SSO/SAML, Organization Management, Admin Panel |
| **Advanced** | Anonymous Users, Impersonation, Email Verification |

---

## Agent Responsibilities

### Rex (Implementation Agent)

**Primary Responsibilities:**
- Initialize BetterAuth in backend
- Configure database adapters
- Set up authentication methods based on PRD
- Implement API routes and middleware

**Standard Implementation:**
```typescript
// auth.ts - Rex's standard setup
import { betterAuth } from "better-auth"
import { prismaAdapter } from "better-auth/adapters/prisma"
import { prisma } from "@/lib/prisma"

export const auth = betterAuth({
  database: prismaAdapter(prisma, {
    provider: "postgresql"
  }),
  emailAndPassword: {
    enabled: true,
    requireEmailVerification: true
  },
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!
    }
  },
  plugins: [] // Add based on requirements
})
```

### Blaze (Frontend Specialist)

**Primary Responsibilities:**
- Implement authentication UI with shadcn/ui
- Create login, signup, profile pages
- Handle auth state management
- Ensure responsive, accessible design

**Standard Implementation:**
```typescript
// app/auth/login/page.tsx - Blaze's UI implementation
import { signIn } from "@/lib/auth-client"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Form } from "@/components/ui/form"

export default function LoginPage() {
  return (
    <div className="container max-w-md mx-auto px-4 py-16">
      <Card>
        <CardHeader>
          <CardTitle>Welcome Back</CardTitle>
          <CardDescription>Sign in to your account</CardDescription>
        </CardHeader>
        <CardContent>
          <AuthForm />
        </CardContent>
      </Card>
    </div>
  )
}
```

### Cleo (Code Quality)

**Validation Checklist:**
- ✅ No hardcoded secrets
- ✅ Proper error handling
- ✅ TypeScript strict mode
- ✅ Environment variables validated
- ✅ CORS properly configured
- ✅ Session security settings correct

### Tess (QA Testing)

**Test Coverage:**
- Registration flow with email verification
- Login with email/password
- OAuth provider flows
- Password reset functionality
- 2FA setup and verification
- Session management and logout
- Authorization and protected routes

---

## Standard Configuration

### Required Environment Variables

```bash
# Database
DATABASE_URL="postgresql://..."

# BetterAuth
BETTER_AUTH_SECRET="..." # 32+ character random string
BETTER_AUTH_URL="http://localhost:3000"

# Email (when required)
EMAIL_FROM="noreply@example.com"
RESEND_API_KEY="..."

# OAuth Providers (as needed)
GITHUB_CLIENT_ID="..."
GITHUB_CLIENT_SECRET="..."
GOOGLE_CLIENT_ID="..."
GOOGLE_CLIENT_SECRET="..."
```

### Database Schema

BetterAuth automatically manages these tables:
- `users` - User accounts
- `sessions` - Active sessions
- `accounts` - OAuth provider accounts
- `verifications` - Email/phone verifications
- `twoFactors` - 2FA configurations

---

## Implementation Patterns

### 1. Basic Email/Password

```typescript
// Minimum viable auth
export const auth = betterAuth({
  database: prismaAdapter(prisma, {
    provider: "postgresql"
  }),
  emailAndPassword: {
    enabled: true
  }
})
```

### 2. With Social Providers

```typescript
// Add OAuth providers
export const auth = betterAuth({
  // ... base config
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!
    },
    google: {
      clientId: process.env.GOOGLE_CLIENT_ID!,
      clientSecret: process.env.GOOGLE_CLIENT_SECRET!
    }
  }
})
```

### 3. Enterprise Features

```typescript
// For B2B applications
export const auth = betterAuth({
  // ... base config
  plugins: [
    twoFactor({
      issuer: "YourApp"
    }),
    organization(),
    admin(),
    multiSession({
      maximumSessions: 5
    })
  ]
})
```

### 4. Consumer Features

```typescript
// For B2C applications
export const auth = betterAuth({
  // ... base config
  plugins: [
    passkey(),
    magicLink({
      sendMagicLink: async ({ email, url }) => {
        // Send email with magic link
      }
    }),
    anonymous()
  ]
})
```

---

## UI Component Patterns

### Login Form (shadcn/ui + BetterAuth)

```typescript
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import * as z from "zod"
import { authClient } from "@/lib/auth-client"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Form, FormControl, FormField, FormItem, FormLabel } from "@/components/ui/form"

const schema = z.object({
  email: z.string().email(),
  password: z.string().min(8)
})

export function LoginForm() {
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema)
  })

  const onSubmit = async (data: z.infer<typeof schema>) => {
    const { error } = await authClient.signIn.email({
      email: data.email,
      password: data.password
    })
    
    if (!error) {
      // Redirect to dashboard
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input {...field} type="email" />
              </FormControl>
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <Input {...field} type="password" />
              </FormControl>
            </FormItem>
          )}
        />
        <Button type="submit" className="w-full">
          Sign In
        </Button>
      </form>
    </Form>
  )
}
```

### Social Login Buttons

```typescript
import { authClient } from "@/lib/auth-client"
import { Button } from "@/components/ui/button"
import { Github, Mail } from "lucide-react"

export function SocialLogins() {
  return (
    <div className="space-y-2">
      <Button
        variant="outline"
        className="w-full"
        onClick={() => authClient.signIn.social({ provider: "github" })}
      >
        <Github className="mr-2 h-4 w-4" />
        Continue with GitHub
      </Button>
      <Button
        variant="outline"
        className="w-full"
        onClick={() => authClient.signIn.social({ provider: "google" })}
      >
        <Mail className="mr-2 h-4 w-4" />
        Continue with Google
      </Button>
    </div>
  )
}
```

---

## Migration from Other Auth Solutions

### From NextAuth.js

```typescript
// Before (NextAuth)
import NextAuth from "next-auth"
import GithubProvider from "next-auth/providers/github"

// After (BetterAuth)
import { betterAuth } from "better-auth"
export const auth = betterAuth({
  socialProviders: {
    github: { /* config */ }
  }
})
```

### From Supabase Auth

```typescript
// Before (Supabase)
const { data, error } = await supabase.auth.signUp({
  email, password
})

// After (BetterAuth)
const { data, error } = await authClient.signUp.email({
  email, password
})
```

### From Auth0

```typescript
// Before (Auth0)
import { useAuth0 } from "@auth0/nextjs-auth0"

// After (BetterAuth)
import { useAuth } from "@/lib/auth-client"
```

---

## Security Best Practices

1. **Always use HTTPS in production**
2. **Set secure session cookies**
3. **Implement rate limiting**
4. **Use strong BETTER_AUTH_SECRET**
5. **Enable email verification for signups**
6. **Implement 2FA for sensitive applications**
7. **Regular security audits with Cipher agent**

---

## Testing Guidelines

### Unit Tests (Rex)

```typescript
describe("Authentication", () => {
  it("should create user with email/password", async () => {
    const result = await auth.api.signUpEmail({
      body: {
        email: "test@example.com",
        password: "securePassword123"
      }
    })
    expect(result.user).toBeDefined()
  })
})
```

### E2E Tests (Tess)

```typescript
test("complete auth flow", async ({ page }) => {
  // Navigate to login
  await page.goto("/auth/login")
  
  // Fill form
  await page.fill('[name="email"]', "test@example.com")
  await page.fill('[name="password"]', "password123")
  
  // Submit
  await page.click('button[type="submit"]')
  
  // Verify redirect
  await expect(page).toHaveURL("/dashboard")
})
```

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "Invalid BETTER_AUTH_SECRET" | Generate with `openssl rand -base64 32` |
| "Database connection failed" | Check DATABASE_URL format |
| "OAuth redirect mismatch" | Update callback URLs in provider settings |
| "Session not persisting" | Check cookie settings and CORS |

---

## Resources

- [BetterAuth Documentation](https://www.better-auth.com)
- [GitHub Repository](https://github.com/better-auth/better-auth)
- [Example Applications](https://github.com/better-auth/better-auth/tree/main/examples)
- [MCP Server Docs](https://www.better-auth.com/docs/mcp)
- [Plugin Directory](https://www.better-auth.com/docs/plugins)

---

## Approval

This document establishes BetterAuth as the official authentication standard for all CTO platform applications, effective immediately.

**Approved by:** CTO Platform Team  
**Date:** November 24, 2025




