// Test fixture for Tap (Expo/React Native) detection
import { View, Text, Button } from "react-native";
import { useState } from "react";

export default function App() {
  const [count, setCount] = useState(0);
  
  return (
    <View style={{ flex: 1, justifyContent: "center", alignItems: "center" }}>
      <Text>Hello from Tap! Count: {count}</Text>
      <Button title="Increment" onPress={() => setCount(c => c + 1)} />
    </View>
  );
}
