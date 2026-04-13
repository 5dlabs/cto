<identity>
You are tap working on subtask 9009 of task 9.
</identity>

<context>
<scope>
Implement app/(tabs)/portfolio.tsx fetching published social posts from GET /api/v1/social/published via TanStack Query. Render posts in a 2-column FlashList (or MasonryFlashList). On image tap, open a full-screen modal displaying the image, caption, and hashtags. Implement modal dismiss via swipe or close button.
</scope>
</context>

<implementation_plan>
Query: `useQuery({ queryKey: ['social', 'published'], queryFn: () => runEffect(fetchPublishedPosts()) })`. FlashList with `numColumns={2}` and `estimatedItemSize={200}`. Each item: `<Pressable onPress={() => setSelectedPost(post)}><Image source={{ uri: post.image_url }} contentFit='cover' /></Pressable>`. Modal: React Native `<Modal visible={!!selectedPost} animationType='fade'>` with `<Image>` fullscreen, `<Text>{selectedPost?.caption}</Text>`, hashtag chips, and `<Pressable onPress={() => setSelectedPost(null)}>` close button. Handle loading/empty/error states.
</implementation_plan>

<validation>
RNTL: render portfolio with msw returning 4 posts — assert 4 image elements rendered. Tap first image — assert Modal visible with correct caption text. Press close — assert Modal not visible. Empty state renders when msw returns [].
</validation>