Implement subtask 9013: Write Jest and React Native Testing Library test suite with >= 80% coverage

## Objective
Write the complete Jest + RNTL test suite covering lib/ (Effect schemas, API functions) and all components/screens. Configure msw for API mocking. Set up jest.config.ts with coverage thresholds. Ensure coverage of happy paths, validation errors, empty states, and error states for all 5 tab screens and the lib/ data layer.

## Steps
Install: @testing-library/react-native, msw@2.x, jest-expo, @testing-library/jest-native. jest.config.ts: preset 'jest-expo', coverageThreshold: { global: { lines: 80, branches: 80 } }, setupFilesAfterFramework: ['@testing-library/jest-native/extend-expect', './jest.setup.ts']. jest.setup.ts: start msw server. Mock modules: 'expo-router' (mock useRouter, useLocalSearchParams), 'expo-notifications', 'expo-camera'. Test files: lib/schemas.test.ts (Schema decode happy/sad), lib/api.test.ts (fetchProducts, fetchAvailability, createOpportunity with msw), components/ProductCard.test.tsx, screens/catalog.test.tsx, screens/detail.test.tsx, screens/quote.test.tsx, screens/chat.test.tsx, screens/portfolio.test.tsx. Run `jest --coverage` to verify threshold.

## Validation
CI run of `jest --coverage` exits 0 with reported line coverage >= 80% and branch coverage >= 80%. All test files pass with zero failing assertions. Coverage report in CI artifacts shows per-file breakdown.