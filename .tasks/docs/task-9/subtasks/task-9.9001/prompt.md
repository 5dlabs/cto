<identity>
You are tap working on subtask 9001 of task 9.
</identity>

<context>
<scope>
Scaffold the Expo SDK 51+ project with TypeScript template at apps/mobile. Install all required dependencies: nativewind@4.x, tailwindcss@3.4, effect@3.x, @effect/schema, @tanstack/react-query v5, expo-router@3.x, expo-notifications, expo-camera, expo-image, @expo/vector-icons, react-native-reanimated, react-native-gesture-handler. Configure babel.config.js for NativeWind and reanimated plugins. Configure metro.config.js for monorepo symlink resolution.
</scope>
</context>

<implementation_plan>
Run `npx create-expo-app@latest apps/mobile --template expo-template-blank-typescript`. Then install all listed packages. In babel.config.js add `plugins: ['nativewind/babel', 'react-native-reanimated/plugin']`. In metro.config.js use `getDefaultConfig` with `resolver.unstable_enablePackageExports: true` and watchFolders pointing to the monorepo root so packages/design-tokens is resolvable. Set `main: 'expo-router/entry'` in package.json. Verify `npx expo start` launches without errors.
</implementation_plan>

<validation>
Run `npx expo start --no-dev` — CLI exits without module-not-found or TypeScript errors. Run `npx tsc --noEmit` — zero type errors on the empty scaffold.
</validation>