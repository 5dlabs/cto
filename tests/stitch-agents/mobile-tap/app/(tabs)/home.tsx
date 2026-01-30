// Code smells for Tap (React Native/Expo) to find:
// - Using web-specific APIs
// - Inline styles instead of StyleSheet
// - Missing accessibility props
// - any types

import React, { useState, useEffect } from 'react';
import { View, Text, TouchableOpacity, ScrollView } from 'react-native';

export default function HomeScreen() {
  const [data, setData] = useState<any>(null);

  useEffect(() => {
    // Using localStorage - doesn't work in React Native!
    const cached = localStorage.getItem('data');
    if (cached) {
      setData(JSON.parse(cached));
    }
  }, []);

  // Inline styles - should use StyleSheet.create
  return (
    <ScrollView style={{ flex: 1, padding: 20 }}>
      <View style={{ marginBottom: 20 }}>
        {/* Missing accessibilityLabel */}
        <TouchableOpacity 
          style={{ backgroundColor: 'blue', padding: 10, borderRadius: 5 }}
          onPress={() => console.log('pressed')}
        >
          <Text style={{ color: 'white', fontSize: 16 }}>Press Me</Text>
        </TouchableOpacity>
      </View>

      {/* Using window object - doesn't exist in RN */}
      <Text style={{ fontSize: 14 }}>
        Width: {window.innerWidth}
      </Text>

      {data && (
        <View style={{ marginTop: 20 }}>
          <Text>{JSON.stringify(data)}</Text>
        </View>
      )}
    </ScrollView>
  );
}
