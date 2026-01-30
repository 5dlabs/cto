// Code smells for Nova to find:
// - any types
// - callback hell
// - no error handling
// - console.log in production code

import { Elysia } from 'elysia';

const app = new Elysia();

// Using 'any' type
function processData(data: any): any {
  console.log("Processing data...");
  return data.value * 2;
}

// Callback hell
app.get('/users', async () => {
  const result = await fetch('/api/users').then(res => {
    return res.json().then(data => {
      return data.map((user: any) => {
        return { ...user, processed: true };
      });
    });
  });
  return result;
});

// No error handling
app.post('/create', async ({ body }) => {
  const data = JSON.parse(body as string);
  const result = processData(data);
  return { success: true, result };
});

// Unused variable
const unusedConfig = {
  port: 3000,
  host: 'localhost'
};

app.listen(8080);
