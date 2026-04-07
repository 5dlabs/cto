Implement subtask 9010: Implement expo-camera barcode scanner screen accessible from equipment tab

## Objective
Create app/equipment/scan.tsx as a barcode scanner screen using expo-camera. Request camera permissions on mount. Render a CameraView with onBarcodeScanned handler. On scan, call GET /api/v1/catalog/products?barcode={code} and navigate to app/equipment/[id].tsx with the matched product ID. Show error if no product found for barcode. Add a 'Scan Barcode' button in the equipment catalog tab header.

## Steps
In scan.tsx: `const [permission, requestPermission] = useCameraPermissions()`. If no permission, show request prompt. `<CameraView style={styles.camera} onBarcodeScanned={handleScan} barcodeScannerSettings={{ barcodeTypes: ['qr', 'code128', 'ean13'] }}>`. In handleScan: debounce to prevent double-fire (scanned ref). Call `runEffect(fetchProductByBarcode(data))` — on success `router.replace('/equipment/' + product.id)`. On no-result show a toast/alert 'Product not found'. Add header right button in index.tsx: `<Stack.Screen options={{ headerRight: () => <Pressable onPress={() => router.push('/equipment/scan')}>` with barcode-outline icon.

## Validation
Jest: mock expo-camera useCameraPermissions returning granted. Mock runEffect returning a product. Call handleScan with barcode data — assert router.replace called with '/equipment/p1'. Mock runEffect returning empty — assert alert shown. Verify 'Scan Barcode' button present in equipment tab header via RNTL.