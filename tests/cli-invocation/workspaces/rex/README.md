# Rex Backend API Demo

A production-grade RESTful API demonstration showcasing backend development best practices for the Rex agent testing.

## Features

✓ RESTful API design with proper HTTP methods and status codes
✓ Comprehensive error handling with custom error classes
✓ Request validation and sanitization
✓ Rate limiting and security headers
✓ CORS support with configurable origins
✓ Request logging for observability
✓ Graceful shutdown handling
✓ Health check and readiness endpoints
✓ Clean architecture with separation of concerns
✓ Environment-based configuration

## Technology Stack

- **Runtime**: Node.js 18+
- **Framework**: Express.js
- **Security**: Helmet, CORS, Rate Limiting
- **Optimization**: Compression

## Quick Start

### Installation

```bash
npm install
```

### Configuration

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your configuration.

### Running the Server

Development mode (with auto-reload):
```bash
npm run dev
```

Production mode:
```bash
npm start
```

The server will start at `http://localhost:3000`

## API Endpoints

See [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) for detailed API documentation.

### Quick Overview

- `GET /health` - Health check
- `GET /readiness` - Readiness probe
- `POST /api/v1/tasks` - Create task
- `GET /api/v1/tasks` - List tasks (with filtering)
- `GET /api/v1/tasks/:id` - Get task by ID
- `PATCH /api/v1/tasks/:id` - Update task
- `DELETE /api/v1/tasks/:id` - Delete task
- `GET /api/v1/tasks/stats` - Get statistics

## Example Usage

### Create a Task

```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Implement user authentication",
    "description": "Add JWT-based authentication",
    "priority": "high",
    "status": "todo"
  }'
```

### Get All Tasks

```bash
curl http://localhost:3000/api/v1/tasks
```

### Filter Tasks

```bash
curl "http://localhost:3000/api/v1/tasks?status=in_progress&priority=high"
```

### Update a Task

```bash
curl -X PATCH http://localhost:3000/api/v1/tasks/task-1 \
  -H "Content-Type: application/json" \
  -d '{
    "status": "completed"
  }'
```

### Get Statistics

```bash
curl http://localhost:3000/api/v1/tasks/stats
```

## Project Structure

```
/workspace
├── src/
│   ├── config/              # Configuration management
│   │   └── index.js
│   ├── controllers/         # Request handlers
│   │   └── taskController.js
│   ├── middleware/          # Custom middleware
│   │   ├── errorHandler.js
│   │   ├── logger.js
│   │   └── validator.js
│   ├── routes/             # API routes
│   │   ├── healthRoutes.js
│   │   └── taskRoutes.js
│   ├── services/           # Business logic
│   │   └── taskService.js
│   ├── app.js              # Express app setup
│   └── index.js            # Server entry point
├── .env.example            # Environment variables template
├── package.json
├── API_DOCUMENTATION.md    # Detailed API docs
└── README.md
```

## Architecture Highlights

### Layered Architecture

1. **Routes Layer**: Defines API endpoints and applies route-specific middleware
2. **Controller Layer**: Handles HTTP requests and responses
3. **Service Layer**: Contains business logic
4. **Middleware Layer**: Request processing, validation, logging, error handling

### Error Handling

- Custom `AppError` class for operational errors
- Global error handling middleware
- Async error wrapper for route handlers
- Different error responses for development and production

### Validation

- Input validation middleware
- Type checking and sanitization
- Clear error messages for invalid requests

### Security

- Helmet for security headers
- CORS configuration
- Rate limiting per IP
- Request size limits
- Input sanitization

## Best Practices Demonstrated

1. **Clean Architecture**: Separation of concerns with clear layers
2. **Error Handling**: Comprehensive error handling strategy
3. **Validation**: Input validation at API boundaries
4. **Security**: Multiple security layers
5. **Observability**: Request logging and health checks
6. **Configuration**: Environment-based configuration management
7. **Graceful Shutdown**: Proper cleanup on termination signals
8. **RESTful Design**: Following REST conventions and HTTP standards
9. **Code Organization**: Logical file structure and naming
10. **Documentation**: Comprehensive API documentation

## MCP Tool Integration Points

This API can be integrated with various MCP tools:

### Kubernetes MCP
Deploy and monitor this API in Kubernetes:
- Health checks for liveness probes
- Readiness checks for readiness probes
- Graceful shutdown for pod termination
- Resource metrics exposed

### Observability MCP
Integration points for monitoring:
- Request logs for analysis
- Error tracking
- Performance metrics
- Health endpoints for uptime monitoring

### GitHub MCP
Version control and CI/CD:
- Automated deployment pipelines
- PR checks and testing
- Release management

## License

MIT
