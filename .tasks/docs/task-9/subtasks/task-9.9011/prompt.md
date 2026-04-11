<identity>
You are tap working on subtask 9011 of task 9.
</identity>

<context>
<scope>
Configure expo-notifications in the root app/_layout.tsx. On app launch, request notification permissions and register for an Expo Push Token. Store the token and POST it to the backend (or log it for manual backend configuration). Set up a notification received handler and a notification response handler that navigates to the quote tab when a quote-status notification is tapped.
</scope>
</context>

<implementation_plan>
In app/_layout.tsx useEffect: `const { status } = await Notifications.requestPermissionsAsync()`. If granted: `const token = (await Notifications.getExpoPushTokenAsync({ projectId: Constants.expoConfig.extra.eas.projectId })).data`. POST token to backend `/api/v1/notifications/register` with device metadata. Set notification handler: `Notifications.setNotificationHandler({ handleNotification: async () => ({ shouldShowAlert: true, shouldPlaySound: true, shouldSetBadge: true }) })`. Add listener: `Notifications.addNotificationResponseReceivedListener(response => { if (response.notification.request.content.data.type === 'quote_status') router.push('/(tabs)/quote') })`. Configure notification channel for Android in Notifications.setNotificationChannelAsync.
</implementation_plan>

<validation>
Mock expo-notifications APIs in Jest. Assert requestPermissionsAsync called on mount. Assert getExpoPushTokenAsync called when permission granted. Assert POST to /api/v1/notifications/register with token value. Simulate notificationResponseReceived event with type='quote_status' — assert router.push called with '/(tabs)/quote'.
</validation>