Implement subtask 9002: Configure NativeWind 4 with shared design tokens from packages/design-tokens

## Objective
Set up NativeWind 4 / Tailwind CSS 3.4 integration. Import the shared tailwind.config.ts from packages/design-tokens as the base preset. Extend in apps/mobile/tailwind.config.ts with NativeWind-specific settings (content paths covering .tsx files). Configure postcss.config.js required by NativeWind 4's build step. Add global.css entry point and import it in the root _layout.tsx.

## Steps
Create apps/mobile/tailwind.config.ts: `import baseConfig from '../../packages/design-tokens/tailwind.config'; export default { presets: [require('nativewind/preset'), baseConfig], content: ['./app/**/*.tsx', './components/**/*.tsx'], };`. Create postcss.config.js: `module.exports = { plugins: { tailwindcss: {}, autoprefixer: {} } };`. Create app/global.css with `@tailwind base; @tailwind components; @tailwind utilities;`. Import `./global.css` in app/_layout.tsx. Verify brand accent color token is available via `className='text-brand-accent'` on a test Text component.

## Validation
Render a Text component with a design-token-derived className (e.g., `className='bg-brand-accent'`). Confirm the correct color appears in Expo Go on iOS simulator. Run `npx tailwindcss --config tailwind.config.ts --content './app/**/*.tsx' --dry-run` — outputs CSS without error.