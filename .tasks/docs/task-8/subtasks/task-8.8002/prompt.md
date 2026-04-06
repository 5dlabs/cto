Implement subtask 8002: Create shared design tokens Tailwind config package at packages/design-tokens

## Objective
Build the shared Tailwind configuration package defining the Sigma-1 brand palette, typography, spacing, and border radius tokens, importable by apps/website.

## Steps
1. Create packages/design-tokens/ with package.json: name @sigma1/design-tokens, type module, main: tailwind.config.ts.
2. Create packages/design-tokens/tailwind.config.ts exporting a Tailwind preset object:
   - colors: sigma1 palette — background: '#0A0A0F', surface: '#12121A', accent: '#00D4FF' (electric cyan), accent-secondary: '#7B2FFF' (purple), text-primary: '#F0F0F5', text-muted: '#8888AA'.
   - fontFamily: { sans: ['Geist', 'Inter', 'sans-serif'] }.
   - borderRadius: { card: '12px', button: '8px', badge: '4px' }.
   - spacing scale: extend with custom values 18, 22, 26 for layout.
3. Add @sigma1/design-tokens as workspace dependency in apps/website/package.json.
4. Update apps/website/tailwind.config.ts to import and spread the preset: `presets: [require('@sigma1/design-tokens')]`.
5. Create packages/design-tokens/tokens.css exporting CSS custom properties matching the token values (for non-Tailwind usage).

## Validation
`cd apps/website && bun run build` succeeds with no Tailwind config errors. Inspect generated CSS: confirm `#00D4FF` appears as a utility class (e.g., `text-accent`). `import preset from '@sigma1/design-tokens'` in a test script resolves without error.