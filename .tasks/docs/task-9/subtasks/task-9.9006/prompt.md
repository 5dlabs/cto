Implement subtask 9006: Implement offline quote queuing and equipment catalog caching

## Objective
Add offline capability: cache equipment catalog locally for offline browsing and queue quote submissions when offline, auto-submitting when connectivity is restored.

## Steps
1. Install `@react-native-community/netinfo` for connectivity detection.
2. Create `lib/offline/connectivityMonitor.ts`: export a `useIsOnline()` hook that reflects current network state.
3. **Equipment caching**: After successful API fetch of categories and products, persist to AsyncStorage (or MMKV if chosen). On app launch or when offline, read from cache. Display a banner indicating 'Viewing cached data' when offline.
4. **Quote queue**: Create `lib/offline/quoteQueue.ts`. When submitting a quote and `isOnline === false`, serialize the quote payload and store in AsyncStorage under a queue key. Show user a 'Quote saved — will submit when online' toast.
5. **Queue processor**: On connectivity restored (NetInfo event), read pending quotes from queue, submit each sequentially, remove from queue on success. Show local notification on successful background submission.
6. Implement queue status indicator in Quote tab: show count of pending offline quotes.
7. Handle edge case: if the same quote is queued multiple times, deduplicate by hash of payload.

## Validation
Mock NetInfo to simulate offline state. Attempt quote submission while offline — verify payload is stored in AsyncStorage queue. Simulate connectivity restored — verify queued quote is submitted via API. Verify equipment catalog renders from cache when offline. Verify 'cached data' banner appears. Deduplicate test: queue same payload twice, verify only one submission.