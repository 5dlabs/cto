---
name: react-native-llm-docs
description: Fetch React Native documentation via llms.txt for up-to-date framework references
agents: [tap, blaze, spark]
triggers: [react native, new architecture, fabric, turbo modules, native components]
llm_docs_url: https://reactnative.dev/llms.txt
---

# React Native LLM Documentation

React Native provides comprehensive LLM-optimized documentation at `https://reactnative.dev/llms.txt`.

## When to Use

Fetch this documentation when:
- Building core React Native components and layouts
- Working with the New Architecture (Fabric, Turbo Modules)
- Creating native modules or components
- Debugging and profiling performance
- Handling platform-specific code (Android/iOS)
- Publishing to app stores

## Key Topics Covered

- **Core Components**: View, Text, Image, FlatList, ScrollView, TextInput
- **APIs**: Animated, Keyboard, Linking, Platform, PermissionsAndroid
- **New Architecture**: Fabric Components, Turbo Modules, Codegen, C++
- **Native Integration**: iOS Native Modules, Android Native Modules
- **Performance**: Profiling, Optimizing FlatList, JavaScript Loading
- **Architecture**: Render Pipeline, Threading Model, View Flattening

## Quick Reference

```typescript
// Fetch React Native docs via Firecrawl
const docs = await firecrawl.scrape({
  url: "https://reactnative.dev/llms.txt",
  formats: ["markdown"]
});
```

## Core Component Reference

| Component | Description |
|-----------|-------------|
| `View` | Container with flexbox layout |
| `Text` | Text display and styling |
| `Image` | Image rendering |
| `FlatList` | Performant list rendering |
| `ScrollView` | Scrollable container |
| `TextInput` | Text input field |
| `Pressable` | Touch handling |
| `Modal` | Overlay content |

## Related Skills

- `expo-llm-docs` - Expo framework documentation
- `better-auth-expo` - Authentication for React Native
