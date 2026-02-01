// Test fixture for Blaze (Next.js) detection
"use client";

import { useState } from "react";

export default function TestPage() {
  const [count, setCount] = useState(0);
  
  return (
    <div>
      <h1>Hello from Blaze!</h1>
      <button onClick={() => setCount(c => c + 1)}>
        Count: {count}
      </button>
    </div>
  );
}
