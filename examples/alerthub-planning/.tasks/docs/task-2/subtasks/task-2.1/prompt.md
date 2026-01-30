# Subtask 2.1: Core Axum Web Server and API Endpoints

## Context
This is a subtask of Task 2. Complete this before moving to dependent subtasks.

## Description
Set up the foundational Axum web server with REST endpoints for notification submission, batch processing, and status queries including request/response models and basic routing structure.

## Implementation Details
Initialize Rust project with Axum framework, create API endpoint handlers for POST /notifications (single submission), POST /notifications/batch (bulk submission), GET /notifications/{id}/status (status queries), and GET /health. Implement request validation, response serialization with serde, error handling middleware, and basic logging setup.

## Dependencies
None (can start immediately)

## Deliverables
1. Implementation code
2. Unit tests
3. Documentation updates
