# Rex Backend API Documentation

## Overview

A production-grade RESTful API for task management built with Node.js and Express, demonstrating backend development best practices.

## Base URL

```
http://localhost:3000/api/v1
```

## Features

- RESTful API design
- Comprehensive error handling
- Request validation
- Rate limiting
- CORS support
- Compression
- Security headers (Helmet)
- Request logging
- Graceful shutdown

## Endpoints

### Health Checks

#### GET /health
Check API health status

**Response:**
```json
{
  "status": "success",
  "message": "API is running",
  "timestamp": "2026-01-31T10:00:00.000Z",
  "uptime": 123.45,
  "environment": "development"
}
```

#### GET /readiness
Check API readiness and resource usage

**Response:**
```json
{
  "status": "ready",
  "checks": {
    "api": "healthy",
    "memory": {
      "used": 45,
      "total": 128,
      "unit": "MB"
    }
  }
}
```

### Tasks

#### POST /api/v1/tasks
Create a new task

**Request Body:**
```json
{
  "title": "Implement user authentication",
  "description": "Add JWT-based authentication",
  "priority": "high",
  "status": "todo"
}
```

**Validation Rules:**
- `title` (required): String, 1-200 characters
- `description` (optional): String, max 2000 characters
- `priority` (optional): One of: `low`, `medium`, `high`, `urgent` (default: `medium`)
- `status` (optional): One of: `todo`, `in_progress`, `completed`, `archived` (default: `todo`)

**Response (201):**
```json
{
  "status": "success",
  "data": {
    "task": {
      "id": "task-1",
      "title": "Implement user authentication",
      "description": "Add JWT-based authentication",
      "priority": "high",
      "status": "todo",
      "createdAt": "2026-01-31T10:00:00.000Z",
      "updatedAt": "2026-01-31T10:00:00.000Z"
    }
  }
}
```

#### GET /api/v1/tasks
Get all tasks with optional filtering

**Query Parameters:**
- `status`: Filter by status
- `priority`: Filter by priority
- `search`: Search in title and description

**Example:** `/api/v1/tasks?status=in_progress&priority=high`

**Response (200):**
```json
{
  "status": "success",
  "results": 2,
  "data": {
    "tasks": [
      {
        "id": "task-1",
        "title": "Implement user authentication",
        "description": "Add JWT-based authentication",
        "priority": "high",
        "status": "todo",
        "createdAt": "2026-01-31T10:00:00.000Z",
        "updatedAt": "2026-01-31T10:00:00.000Z"
      }
    ]
  }
}
```

#### GET /api/v1/tasks/:id
Get a specific task by ID

**Response (200):**
```json
{
  "status": "success",
  "data": {
    "task": {
      "id": "task-1",
      "title": "Implement user authentication",
      "description": "Add JWT-based authentication",
      "priority": "high",
      "status": "todo",
      "createdAt": "2026-01-31T10:00:00.000Z",
      "updatedAt": "2026-01-31T10:00:00.000Z"
    }
  }
}
```

**Error (404):**
```json
{
  "status": "fail",
  "message": "Task not found"
}
```

#### PATCH /api/v1/tasks/:id
Update a task

**Request Body:** (at least one field required)
```json
{
  "title": "Updated title",
  "status": "in_progress"
}
```

**Response (200):**
```json
{
  "status": "success",
  "data": {
    "task": {
      "id": "task-1",
      "title": "Updated title",
      "description": "Add JWT-based authentication",
      "priority": "high",
      "status": "in_progress",
      "createdAt": "2026-01-31T10:00:00.000Z",
      "updatedAt": "2026-01-31T10:15:00.000Z"
    }
  }
}
```

#### DELETE /api/v1/tasks/:id
Delete a task

**Response (204):**
No content

#### GET /api/v1/tasks/stats
Get task statistics

**Response (200):**
```json
{
  "status": "success",
  "data": {
    "stats": {
      "total": 10,
      "byStatus": {
        "todo": 3,
        "in_progress": 4,
        "completed": 2,
        "archived": 1
      },
      "byPriority": {
        "low": 2,
        "medium": 5,
        "high": 2,
        "urgent": 1
      }
    }
  }
}
```

## Error Responses

All errors follow a consistent format:

**Client Error (4xx):**
```json
{
  "status": "fail",
  "message": "Error description"
}
```

**Server Error (5xx):**
```json
{
  "status": "error",
  "message": "Something went wrong"
}
```

## Rate Limiting

- Window: 15 minutes
- Max requests: 100 per IP
- Headers: `RateLimit-*` headers included in responses

## Security Features

- Helmet.js for security headers
- CORS configuration
- Rate limiting
- Request size limits (10kb)
- Input validation and sanitization

## Architecture

```
src/
├── config/           # Configuration management
├── controllers/      # Request handlers
├── middleware/       # Custom middleware
├── routes/          # API routes
├── services/        # Business logic
├── app.js           # Express app setup
└── index.js         # Server entry point
```

## Best Practices Demonstrated

1. **Separation of Concerns**: Controllers, services, and routes are separated
2. **Error Handling**: Centralized error handling with custom error classes
3. **Validation**: Input validation middleware
4. **Security**: Multiple security layers
5. **Logging**: Request logging for observability
6. **Configuration**: Environment-based configuration
7. **Graceful Shutdown**: Proper cleanup on shutdown signals
8. **RESTful Design**: Following REST conventions
