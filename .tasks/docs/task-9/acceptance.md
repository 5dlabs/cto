## Acceptance Criteria

- [ ] 1. Component tests (Jest + React Native Testing Library): ProductCard renders correctly with mock data; ChatBubble displays message text and timestamp; QuoteLineItem shows product name and quantity. 2. Navigation test: verify tab navigation renders correct screens; verify Equipment tab → Product detail deep link works. 3. Chat flow test: mock WebSocket, send message from Chat tab, verify message appears in conversation; verify incoming message triggers notification mock. 4. Barcode scanner test: mock camera barcode detection event, verify RMS API called with scanned code, verify equipment details screen shown. 5. Quote builder test: complete all steps with mock data, verify API submission payload contains correct products, dates, and venue. 6. Offline test: enable airplane mode mock, add items to quote, verify queued; disable airplane mode, verify submission fires. 7. EAS build: `eas build --platform all --profile preview` completes successfully. 8. Visual regression: screenshot comparison of key screens at iPhone SE (375px) and iPhone 15 Pro (393px) widths.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.