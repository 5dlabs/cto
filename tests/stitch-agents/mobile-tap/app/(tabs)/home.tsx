/**
 * Stitch test fixture for Tap (Expo/React Native mobile agent)
 * 
 * This file contains intentional issues for testing remediation:
 * - Missing key prop in FlatList
 * - Inline styles (should use StyleSheet)
 * - Type errors
 */

import { useState, useEffect } from 'react'
import { View, Text, FlatList, TouchableOpacity, TextInput } from 'react-native'
import { useQuery } from '@tanstack/react-query'

interface User {
  id: number
  name: string
  email: string
}

// TODO: Intentional issue - any type
async function fetchUsers(): Promise<any> {
  const response = await fetch('https://api.example.com/users')
  return response.json()
}

export default function HomeScreen() {
  const [search, setSearch] = useState('')
  
  // TODO: Intentional issue - console.log in production code
  useEffect(() => {
    console.log('HomeScreen mounted')
  }, [])

  const { data: users, isLoading } = useQuery({
    queryKey: ['users'],
    queryFn: fetchUsers,
  })

  // TODO: Intentional issue - inline styles instead of StyleSheet
  return (
    <View style={{ flex: 1, padding: 16, backgroundColor: '#fff' }}>
      <TextInput
        value={search}
        onChangeText={setSearch}
        placeholder="Search users..."
        style={{ 
          borderWidth: 1, 
          borderColor: '#ccc', 
          padding: 12, 
          borderRadius: 8,
          marginBottom: 16 
        }}
      />
      
      {isLoading ? (
        <Text>Loading...</Text>
      ) : (
        // TODO: Intentional issue - missing keyExtractor
        <FlatList
          data={users}
          renderItem={({ item }: { item: User }) => (
            <TouchableOpacity 
              style={{ padding: 16, borderBottomWidth: 1, borderBottomColor: '#eee' }}
            >
              <Text style={{ fontSize: 16, fontWeight: 'bold' }}>{item.name}</Text>
              <Text style={{ color: '#666' }}>{item.email}</Text>
            </TouchableOpacity>
          )}
        />
      )}
    </View>
  )
}
