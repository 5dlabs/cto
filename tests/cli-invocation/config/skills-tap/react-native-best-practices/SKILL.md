---
name: react-native-best-practices
description: React Native performance optimization - FPS, bundle size, TTI, memory management, animations.
---

# React Native Best Practices

Performance optimization guide for React Native applications based on Callstack's "Ultimate Guide to React Native Optimization".

## When to Apply

Reference these guidelines when:
- Debugging slow/janky UI or animations
- Investigating memory leaks (JS or native)
- Optimizing app startup time (TTI)
- Reducing bundle or app size
- Writing native modules (Turbo Modules)
- Profiling React Native performance

## Priority-Ordered Guidelines

| Priority | Category | Impact | Prefix |
|----------|----------|--------|--------|
| 1 | FPS & Re-renders | CRITICAL | `js-*` |
| 2 | Bundle Size | CRITICAL | `bundle-*` |
| 3 | TTI Optimization | HIGH | `native-*`, `bundle-*` |
| 4 | Native Performance | HIGH | `native-*` |
| 5 | Memory Management | MEDIUM-HIGH | `js-*`, `native-*` |
| 6 | Animations | MEDIUM | `js-*` |

## Quick Reference

### Critical: FPS & Re-renders

**Profile first:**
```bash
# Open React Native DevTools
# Press 'j' in Metro, or shake device → "Open DevTools"
```

**Common fixes:**
- Replace ScrollView with FlatList/FlashList for lists
- Use React Compiler for automatic memoization
- Use atomic state (Jotai/Zustand) to reduce re-renders
- Use `useDeferredValue` for expensive computations

### Critical: Bundle Size

**Analyze bundle:**
```bash
npx react-native bundle \
  --entry-file index.js \
  --bundle-output output.js \
  --platform ios \
  --sourcemap-output output.js.map \
  --dev false --minify true

npx source-map-explorer output.js --no-border-checks
```

**Common fixes:**
- Avoid barrel imports (import directly from source)
- Remove unnecessary Intl polyfills (Hermes has native support)
- Enable tree shaking (Expo SDK 52+ or Re.Pack)
- Enable R8 for Android native code shrinking

### High: TTI Optimization

**Measure TTI:**
- Use `react-native-performance` for markers
- Only measure cold starts (exclude warm/hot/prewarm)

**Common fixes:**
- Disable JS bundle compression on Android (enables Hermes mmap)
- Use native navigation (react-native-screens)
- Defer non-critical work with `InteractionManager`

### High: Native Performance

**Profile native:**
- iOS: Xcode Instruments → Time Profiler
- Android: Android Studio → CPU Profiler

**Common fixes:**
- Use background threads for heavy native work
- Prefer async over sync Turbo Module methods
- Use C++ for cross-platform performance-critical code

## Problem → Solution Mapping

| Problem | Start With |
|---------|------------|
| App feels slow/janky | Measure FPS → Profile React |
| Too many re-renders | Profile React → React Compiler |
| Slow startup (TTI) | Measure TTI → Analyze bundle |
| Large app size | Analyze app → Enable R8 |
| Memory growing | Check JS or native memory leaks |
| Animation drops frames | Use Reanimated worklets |
| List scroll jank | Use FlatList/FlashList |
| TextInput lag | Use uncontrolled components |
| Native module slow | Turbo Modules → Threading |

## Key Patterns

### Use FlashList for Large Lists

```tsx
// Bad: ScrollView with many items
<ScrollView>
  {items.map(item => <Item key={item.id} {...item} />)}
</ScrollView>

// Good: FlashList with virtualization
import { FlashList } from "@shopify/flash-list";

<FlashList
  data={items}
  renderItem={({ item }) => <Item {...item} />}
  estimatedItemSize={100}
/>
```

### Avoid Barrel Imports

```tsx
// Bad: imports entire barrel
import { Button, Card, Modal } from '@/components';

// Good: direct imports
import { Button } from '@/components/Button';
import { Card } from '@/components/Card';
```

### Use Reanimated for Smooth Animations

```tsx
import Animated, { 
  useAnimatedStyle, 
  withSpring 
} from 'react-native-reanimated';

const animatedStyle = useAnimatedStyle(() => ({
  transform: [{ scale: withSpring(pressed.value ? 0.9 : 1) }],
}));
```

### Defer Non-Critical Work

```tsx
import { InteractionManager } from 'react-native';

useEffect(() => {
  InteractionManager.runAfterInteractions(() => {
    // Expensive work after animations complete
    loadAnalytics();
  });
}, []);
```

## Attribution

Based on "The Ultimate Guide to React Native Optimization" by [Callstack](https://github.com/callstackincubator/agent-skills) - 642+ installs.
