/**
 * API Integration Tests
 * Tests core API functionality including authentication and task management
 */

const { test, describe, before, after } = require('node:test');
const assert = require('node:assert');
const http = require('node:http');

const BASE_URL = 'http://localhost:3000';
let server;
let authToken;

// HTTP request helper
async function request(method, path, body = null, headers = {}) {
  return new Promise((resolve, reject) => {
    const options = {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...headers,
      },
    };

    const req = http.request(`${BASE_URL}${path}`, options, (res) => {
      let data = '';
      res.on('data', (chunk) => (data += chunk));
      res.on('end', () => {
        try {
          const parsed = data ? JSON.parse(data) : null;
          resolve({ status: res.statusCode, body: parsed, headers: res.headers });
        } catch (e) {
          resolve({ status: res.statusCode, body: data, headers: res.headers });
        }
      });
    });

    req.on('error', reject);
    if (body) {
      req.write(JSON.stringify(body));
    }
    req.end();
  });
}

describe('Rex Backend API Tests', () => {
  before(async () => {
    // Server should already be running
    console.log('Starting API tests...');
  });

  after(async () => {
    console.log('API tests completed');
  });

  describe('Health Check Endpoints', () => {
    test('GET /health should return API health status', async () => {
      const res = await request('GET', '/health');

      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.status, 'success');
      assert.ok(res.body.timestamp);
      assert.ok(typeof res.body.uptime === 'number');
      assert.strictEqual(res.body.environment, 'development');
    });

    test('GET /readiness should return readiness status', async () => {
      const res = await request('GET', '/readiness');

      assert.strictEqual(res.status, 200);
      assert.ok(['ready', 'not ready'].includes(res.body.status));
      assert.ok(res.body.checks);
    });
  });

  describe('Authentication Endpoints', () => {
    test('POST /api/v1/auth/login should authenticate user', async () => {
      const res = await request('POST', '/api/v1/auth/login', {
        username: 'admin',
        password: 'admin123',
      });

      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.status, 'success');
      assert.ok(res.body.data.token);
      assert.strictEqual(res.body.data.user.username, 'admin');
      assert.strictEqual(res.body.data.user.role, 'admin');

      // Store token for later tests
      authToken = res.body.data.token;
    });

    test('POST /api/v1/auth/login should reject invalid credentials', async () => {
      const res = await request('POST', '/api/v1/auth/login', {
        username: 'admin',
        password: 'wrongpassword',
      });

      assert.strictEqual(res.status, 401);
      assert.strictEqual(res.body.status, 'fail');
    });

    test('POST /api/v1/auth/register should create new user', async () => {
      const res = await request('POST', '/api/v1/auth/register', {
        username: `testuser_${Date.now()}`,
        password: 'test123',
        email: 'test@example.com',
      });

      assert.strictEqual(res.status, 201);
      assert.strictEqual(res.body.status, 'success');
      assert.ok(res.body.data.token);
      assert.ok(res.body.data.user);
    });

    test('GET /api/v1/auth/me should return current user', async () => {
      const res = await request('GET', '/api/v1/auth/me', null, {
        Authorization: `Bearer ${authToken}`,
      });

      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.status, 'success');
      assert.strictEqual(res.body.data.user.username, 'admin');
    });

    test('GET /api/v1/auth/me should reject without token', async () => {
      const res = await request('GET', '/api/v1/auth/me');

      assert.strictEqual(res.status, 401);
    });
  });

  describe('Task Management Endpoints', () => {
    let taskId;

    test('POST /api/v1/tasks should create a new task', async () => {
      const res = await request('POST', '/api/v1/tasks', {
        title: 'Test Task',
        description: 'This is a test task',
        priority: 'high',
        status: 'todo',
      });

      assert.strictEqual(res.status, 201);
      assert.strictEqual(res.body.status, 'success');
      assert.strictEqual(res.body.data.task.title, 'Test Task');
      assert.strictEqual(res.body.data.task.priority, 'high');
      assert.ok(res.body.data.task.id);

      taskId = res.body.data.task.id;
    });

    test('GET /api/v1/tasks should return all tasks', async () => {
      const res = await request('GET', '/api/v1/tasks');

      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.status, 'success');
      assert.ok(Array.isArray(res.body.data.tasks));
      assert.ok(res.body.results >= 1);
    });

    test('GET /api/v1/tasks/:id should return specific task', async () => {
      const res = await request('GET', `/api/v1/tasks/${taskId}`);

      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.data.task.id, taskId);
    });

    test('PATCH /api/v1/tasks/:id should update task', async () => {
      const res = await request('PATCH', `/api/v1/tasks/${taskId}`, {
        status: 'in_progress',
      });

      assert.strictEqual(res.status, 200);
      assert.strictEqual(res.body.data.task.status, 'in_progress');
    });

    test('GET /api/v1/tasks?status=in_progress should filter tasks', async () => {
      const res = await request('GET', '/api/v1/tasks?status=in_progress');

      assert.strictEqual(res.status, 200);
      assert.ok(res.body.data.tasks.every(t => t.status === 'in_progress'));
    });

    test('GET /api/v1/tasks/stats should return statistics', async () => {
      const res = await request('GET', '/api/v1/tasks/stats');

      assert.strictEqual(res.status, 200);
      assert.ok(res.body.data.stats.total >= 1);
      assert.ok(res.body.data.stats.byStatus);
      assert.ok(res.body.data.stats.byPriority);
    });

    test('DELETE /api/v1/tasks/:id should delete task', async () => {
      const res = await request('DELETE', `/api/v1/tasks/${taskId}`);

      assert.strictEqual(res.status, 204);
    });

    test('GET /api/v1/tasks/:id should return 404 for deleted task', async () => {
      const res = await request('GET', `/api/v1/tasks/${taskId}`);

      assert.strictEqual(res.status, 404);
    });
  });

  describe('Input Validation', () => {
    test('POST /api/v1/tasks should reject missing title', async () => {
      const res = await request('POST', '/api/v1/tasks', {
        description: 'No title',
      });

      assert.strictEqual(res.status, 400);
    });

    test('POST /api/v1/tasks should reject invalid priority', async () => {
      const res = await request('POST', '/api/v1/tasks', {
        title: 'Test',
        priority: 'invalid',
      });

      assert.strictEqual(res.status, 400);
    });

    test('POST /api/v1/tasks should reject invalid status', async () => {
      const res = await request('POST', '/api/v1/tasks', {
        title: 'Test',
        status: 'invalid',
      });

      assert.strictEqual(res.status, 400);
    });
  });

  describe('Error Handling', () => {
    test('GET /nonexistent should return 404', async () => {
      const res = await request('GET', '/nonexistent');

      assert.strictEqual(res.status, 404);
      assert.strictEqual(res.body.status, 'fail');
    });

    test('Rate limit headers should be present', async () => {
      const res = await request('GET', '/health');

      assert.ok(res.headers['ratelimit-limit']);
      assert.ok(res.headers['ratelimit-remaining']);
    });
  });
});
