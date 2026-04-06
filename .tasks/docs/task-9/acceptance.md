## Acceptance Criteria

- [ ] 1. expo start --no-dev builds without TypeScript errors and Expo Go app launches on iOS simulator and Android emulator. 2. Equipment tab: renders FlatList with product cards; search input filters results in real time (Jest + React Native Testing Library: render, type in search input, assert filtered items). 3. Product detail: date range selection triggers GET availability call (mock API with msw); quantity_available badge updates from skeleton to numeric value. 4. Quote builder: submitting form with empty required fields shows validation error messages (Effect.Schema decoding failure triggers inline error UI). 5. Successful quote submission (mocked 201 response) shows confirmation screen with quote_id displayed. 6. Push notification: call Expo Push API with test token; device displays notification with title containing opportunity status. 7. jest --coverage reports >= 80% coverage across lib/ and components/. 8. EAS build preview profile produces APK/IPA successfully in CI without build errors.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.