<identity>
You are tap working on subtask 9003 of task 9.
</identity>

<context>
<scope>
Create the full Expo Router directory structure: app/_layout.tsx (root layout with QueryClientProvider and TanStack Query setup), app/(tabs)/_layout.tsx (Tabs navigator with 4 tab entries), app/(tabs)/index.tsx, app/(tabs)/quote.tsx, app/(tabs)/chat.tsx, app/(tabs)/portfolio.tsx, and app/equipment/[id].tsx as placeholder screens. Configure tab icons using Ionicons and active tab color from the brand accent token.
</scope>
</context>

<implementation_plan>
app/_layout.tsx: wrap with `<QueryClientProvider client={queryClient}><Stack /></QueryClientProvider>`. app/(tabs)/_layout.tsx: `<Tabs screenOptions={{ tabBarActiveTintColor: tokens.brandAccent }}><Tabs.Screen name='index' options={{ title: 'Equipment', tabBarIcon: ({color}) => <Ionicons name='search' color={color} /> }} />` — repeat for quote (document-text), chat (chatbubble), portfolio (images). Each tab screen file exports a basic `<View><Text>Placeholder</Text></View>`. app/equipment/[id].tsx returns placeholder. Verify tab bar renders with correct icons and active highlight.
</implementation_plan>

<validation>
Open app in Expo Go — bottom tab bar displays 4 tabs with correct Ionicons icons. Tapping each tab navigates to the corresponding screen without crash. Active tab icon color matches brand accent from design tokens.
</validation>