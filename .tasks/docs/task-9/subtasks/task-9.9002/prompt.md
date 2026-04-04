Implement subtask 9002: Port design system tokens and build shared component library

## Objective
Port the web frontend's design tokens (colors, typography, spacing) to React Native using NativeWind or chosen styling approach. Build the shared component library: ProductCard, AvailabilityBadge, QuoteLineItem, ChatBubble.

## Steps
1. Install NativeWind v4: `npx expo install nativewind tailwindcss` and configure `tailwind.config.js` to match web design tokens (dark theme colors, spacing scale, border radii).
2. Create `lib/tokens.ts` exporting color palette, typography scale, and spacing values mirrored from the web Task 8 design system.
3. Install `expo-font` and load custom brand fonts; configure fallback fonts.
4. Build `components/ProductCard.tsx`: product image (expo-image), name, price, AvailabilityBadge. Accept `onPress` for navigation.
5. Build `components/AvailabilityBadge.tsx`: colored badge showing 'Available', 'Limited', 'Unavailable' with appropriate brand colors.
6. Build `components/QuoteLineItem.tsx`: product name, quantity stepper, date range display, remove action.
7. Build `components/ChatBubble.tsx`: message text, timestamp, sent/received styling differentiation, support for rich content slot (e.g., embedded ProductCard).
8. Build common UI primitives: `Button`, `TextInput`, `Card`, `Badge`, `LoadingSpinner`, `EmptyState`.
9. Ensure all components handle dark theme as default, matching web visual language.

## Validation
Jest + React Native Testing Library: Render each component with mock props and snapshot test. ProductCard renders image, name, price, and badge. ChatBubble differentiates sent vs received styling. QuoteLineItem quantity stepper increments/decrements correctly. Visual check on iOS Simulator at iPhone SE and iPhone 15 Pro widths.