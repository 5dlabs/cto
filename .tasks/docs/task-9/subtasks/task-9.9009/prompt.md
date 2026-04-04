Implement subtask 9009: Implement Barcode Scan tab with camera scanner and RMS integration

## Objective
Build the Scan tab with camera-based barcode detection using expo-camera, integration with RMS ScanBarcode API, equipment details display, and check-in/check-out actions with haptic feedback.

## Steps
1. Install `expo-camera` and `expo-haptics`.
2. Build `screens/scan/ScanScreen.tsx`: Full-screen camera viewfinder with barcode scanning overlay (targeting area indicator). Request camera permissions on first access.
3. Configure barcode types to detect: Code128, QR, EAN13 (based on RMS barcode format).
4. On barcode detected: trigger haptic feedback (`Haptics.notificationAsync(Success)`), pause scanner, call `POST /api/v1/inventory/scan` with scanned barcode value.
5. Build `screens/scan/EquipmentDetailSheet.tsx`: Bottom sheet or modal displaying scanned equipment details (name, serial number, current status, last location, assigned event).
6. Add action buttons in detail sheet: 'Check Out' (assign to event/user) and 'Check In' (return to inventory). Each calls respective RMS API endpoint.
7. Handle scan errors: invalid barcode (not found in RMS) shows 'Equipment not found' state. Network error shows retry option.
8. Add manual entry option: text input for typing barcode number if camera scan fails.
9. Handle camera permission denied: show explanation and settings link.

## Validation
Mock expo-camera barcode detection event with a test barcode value. Verify RMS scan API is called with the correct barcode. Mock successful API response, verify equipment detail sheet renders with name, status, serial number. Verify 'Check Out' button calls check-out API. Verify haptic feedback triggered on scan. Mock 'not found' API response, verify error state renders. Manual entry test: type barcode, submit, verify same API flow.