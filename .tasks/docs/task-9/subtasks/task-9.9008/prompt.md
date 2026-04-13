<identity>
You are tap working on subtask 9008 of task 9.
</identity>

<context>
<scope>
Implement app/(tabs)/chat.tsx as a native chat UI: a FlatList of messages (user/assistant bubbles) and a TextInput + Send button at the bottom. On send, call POST /api/v1/chat via useMutation wrapping Effect fetchChat. Append assistant response to message list. Maintain conversation history in component state for context. Implement keyboard-avoiding behavior with KeyboardAvoidingView.
</scope>
</context>

<implementation_plan>
Message state: `const [messages, setMessages] = useState<Message[]>([])`. On send: `mutation.mutateAsync({ messages: [...messages, { role: 'user', content: input }] })` — on success append `{ role: 'assistant', content: result.content }`. FlatList inverted for chat-style scroll. MessageBubble: user messages right-aligned with brand accent bg, assistant messages left-aligned with neutral bg. KeyboardAvoidingView behavior='padding' on iOS, 'height' on Android. TextInput clears after send. Show typing indicator (animated dots) while mutation isPending.
</implementation_plan>

<validation>
RNTL: render chat screen with msw intercepting POST /api/v1/chat returning { content: 'Hello!' }. Type 'Hi' in input, press Send — assert user bubble 'Hi' appears. Assert assistant bubble 'Hello!' appears after mock resolves. Assert input cleared. Assert full conversation history sent in subsequent request body.
</validation>