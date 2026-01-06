---
name: expo-patterns
description: Expo/React Native patterns including expo-router, EAS build/deploy, and cross-platform mobile development.
agents: [tap]
triggers: [expo, react native, mobile, ios, android, eas]
---

# Expo Mobile Patterns

Cross-platform mobile development with Expo, covering navigation, builds, and native integrations.

## Core Stack

| Component | Library | Purpose |
|-----------|---------|---------|
| Framework | Expo SDK | Native modules |
| Navigation | Expo Router | File-based routing |
| CLI | Expo CLI / EAS CLI | Development and builds |
| Build | EAS Build | Cloud builds |
| Deploy | EAS Submit/Update | Store submission, OTA updates |
| State | TanStack Query, Zustand | Server and client state |
| Animation | Reanimated | 60fps native animations |
| Gestures | Gesture Handler | Native touch handling |

## Context7 Library IDs

Query these libraries for current best practices:

- **Better Auth**: `/better-auth/better-auth`
- **Expo**: `expo`
- **React Native**: `/facebook/react-native`

## Expo Router Navigation

### Root Layout

```typescript
// app/_layout.tsx - Root layout with Stack
import { Stack } from 'expo-router';
export default function RootLayout() {
  return <Stack />;
}
```

### Tab Navigation

```typescript
// app/(tabs)/_layout.tsx - Tab navigation
import { Tabs } from 'expo-router';
export default function TabLayout() {
  return <Tabs />;
}
```

### Dynamic Routes

```typescript
// app/[id].tsx - Dynamic routes
import { useLocalSearchParams } from 'expo-router';
export default function Detail() {
  const { id } = useLocalSearchParams();
  return <View>...</View>;
}
```

### Navigation

```typescript
// Navigation with Link
import { Link } from 'expo-router';
<Link href="/profile">Go to Profile</Link>

// Programmatic navigation
import { router } from 'expo-router';
router.push('/settings');
router.replace('/home');
router.back();
```

## App Configuration

```typescript
// app.config.ts - Dynamic configuration
export default {
  expo: {
    name: "MyApp",
    slug: "my-app",
    version: "1.0.0",
    ios: {
      bundleIdentifier: "com.company.myapp",
      supportsTablet: true,
    },
    android: {
      package: "com.company.myapp",
      adaptiveIcon: {
        foregroundImage: "./assets/adaptive-icon.png",
        backgroundColor: "#ffffff"
      }
    },
    plugins: [
      "expo-router",
      ["expo-splash-screen", { backgroundColor: "#ffffff" }],
      ["expo-camera", { cameraPermission: "Allow camera access" }]
    ]
  }
};
```

## EAS Build Configuration

### eas.json

```json
{
  "build": {
    "development": {
      "developmentClient": true,
      "distribution": "internal",
      "ios": { "simulator": true }
    },
    "preview": {
      "distribution": "internal"
    },
    "production": {
      "autoIncrement": true
    }
  },
  "submit": {
    "production": {
      "ios": { "appleId": "...", "ascAppId": "..." },
      "android": { "track": "internal" }
    }
  }
}
```

### Build Commands

```bash
# Install EAS CLI
npm install -g eas-cli
eas login

# Configure project
eas build:configure

# Development build (includes dev tools)
eas build --platform ios --profile development
eas build --platform android --profile development

# Production build
eas build --platform all --profile production

# Submit to stores
eas submit --platform ios
eas submit --platform android

# OTA Update (no app store review needed!)
eas update --branch production --message "Bug fix"
```

## Common Expo SDK Patterns

```typescript
// Camera
import { CameraView, useCameraPermissions } from 'expo-camera';

// Notifications  
import * as Notifications from 'expo-notifications';

// Location
import * as Location from 'expo-location';

// File System
import * as FileSystem from 'expo-file-system';

// Secure Storage
import * as SecureStore from 'expo-secure-store';

// Haptics
import * as Haptics from 'expo-haptics';

// Splash Screen control
import * as SplashScreen from 'expo-splash-screen';
SplashScreen.preventAutoHideAsync();
// After loading...
SplashScreen.hideAsync();
```

## Reanimated Animations

```typescript
import Animated, { 
  useSharedValue, 
  useAnimatedStyle, 
  withSpring 
} from 'react-native-reanimated';

function AnimatedBox() {
  const offset = useSharedValue(0);
  
  const animatedStyles = useAnimatedStyle(() => ({
    transform: [{ translateX: withSpring(offset.value * 255) }],
  }));
  
  return (
    <Animated.View style={[styles.box, animatedStyles]} />
  );
}
```

## Gesture Handling

```typescript
import { GestureDetector, Gesture } from 'react-native-gesture-handler';

function SwipeableCard() {
  const gesture = Gesture.Pan()
    .onUpdate((event) => {
      offset.value = event.translationX;
    })
    .onEnd(() => {
      offset.value = withSpring(0);
    });

  return (
    <GestureDetector gesture={gesture}>
      <Animated.View style={animatedStyles} />
    </GestureDetector>
  );
}
```

## Environment Variables

```bash
# .env files for local dev
EXPO_PUBLIC_API_URL=https://api.example.com

# Access in code (EXPO_PUBLIC_ prefix required)
const apiUrl = process.env.EXPO_PUBLIC_API_URL;

# EAS environments: development, preview, production
eas env:pull --environment development
```

## Validation Commands

```bash
# Type check
npx tsc --noEmit

# Lint
npx eslint .

# Tests
npm test

# Doctor check
npx expo-doctor

# Start dev
npx expo start
```

## Mobile Best Practices

- **Use Expo Router** for navigation (file-based, automatic deep linking)
- **Respect platform conventions** (iOS HIG, Material Design)
- **60fps animations** with Reanimated worklets
- **Handle safe areas** with `react-native-safe-area-context`
- **Offline-first** with proper loading/error states
- **Test on real devices** via EAS internal distribution
- **Accessibility** - use `accessibilityLabel`, `accessibilityRole`
- **Deep linking** - automatic with Expo Router
- **OTA updates** - use EAS Update for instant bug fixes
