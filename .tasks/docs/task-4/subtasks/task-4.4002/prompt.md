Implement subtask 4002: Initialize shadcn/ui and install required components

## Objective
Run `shadcn-ui init` to set up the component system with Tailwind CSS, then install all needed components (Card, Badge, Tabs, Dialog, Skeleton, Button, Table) into `components/ui/`.

## Steps
1. Run `npx shadcn-ui@latest init` in the project root:
   - Select Tailwind CSS as the styling approach
   - Configure `components.json` with the correct paths: `components` alias pointing to `@/components`, `utils` to `@/lib/utils`
   - Confirm `tailwind.config.ts` is updated with shadcn's required config (CSS variables, animation plugin)
2. Install each component individually:
   - `npx shadcn-ui@latest add card` → `components/ui/card.tsx`
   - `npx shadcn-ui@latest add badge` → `components/ui/badge.tsx`
   - `npx shadcn-ui@latest add tabs` → `components/ui/tabs.tsx`
   - `npx shadcn-ui@latest add dialog` → `components/ui/dialog.tsx`
   - `npx shadcn-ui@latest add skeleton` → `components/ui/skeleton.tsx`
   - `npx shadcn-ui@latest add button` → `components/ui/button.tsx`
   - `npx shadcn-ui@latest add table` → `components/ui/table.tsx`
3. Verify `lib/utils.ts` exists with the `cn()` utility function (class merging via clsx + tailwind-merge).
4. Confirm all components are copied into the codebase (no runtime external dependency).
5. Run a quick build (`next build`) to ensure no import errors or Tailwind config issues.

## Validation
Verify each component file exists at its expected path under `components/ui/`. Import each component in a test file and confirm it renders without errors. Run `next build` — zero errors related to shadcn components or missing utilities. Confirm `cn()` function works correctly by passing test class strings.