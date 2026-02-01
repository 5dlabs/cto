"""
Test suite for REX Backend API Demo
"""

import pytest
from fastapi.testclient import TestClient
from api_service import app, in_memory_db, User

client = TestClient(app)


def setup_function():
    """Reset the in-memory database before each test"""
    in_memory_db.clear()


def test_health_check():
    """Test health check endpoint"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert "timestamp" in data


def test_metrics_endpoint():
    """Test metrics endpoint"""
    response = client.get("/metrics")
    assert response.status_code == 200
    data = response.json()
    assert "total_users" in data
    assert data["total_users"] == 0


def test_create_user():
    """Test user creation"""
    user_data = {
        "username": "testuser",
        "email": "test@example.com"
    }
    response = client.post("/api/v1/users", json=user_data)
    assert response.status_code == 201
    data = response.json()
    assert data["username"] == "testuser"
    assert data["email"] == "test@example.com"
    assert "id" in data
    assert "created_at" in data


def test_get_users():
    """Test retrieving all users"""
    client.post("/api/v1/users", json={"username": "user1", "email": "user1@example.com"})
    client.post("/api/v1/users", json={"username": "user2", "email": "user2@example.com"})

    response = client.get("/api/v1/users")
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 2


def test_get_user_by_id():
    """Test retrieving a specific user"""
    create_response = client.post("/api/v1/users", json={"username": "testuser", "email": "test@example.com"})
    user_id = create_response.json()["id"]

    response = client.get(f"/api/v1/users/{user_id}")
    assert response.status_code == 200
    data = response.json()
    assert data["id"] == user_id


def test_get_nonexistent_user():
    """Test retrieving a user that doesn't exist"""
    response = client.get("/api/v1/users/999")
    assert response.status_code == 404


def test_duplicate_username():
    """Test that duplicate usernames are rejected"""
    client.post("/api/v1/users", json={"username": "testuser", "email": "test1@example.com"})
    response = client.post("/api/v1/users", json={"username": "testuser", "email": "test2@example.com"})
    assert response.status_code == 400
    assert "Username already exists" in response.json()["detail"]


def test_duplicate_email():
    """Test that duplicate emails are rejected"""
    client.post("/api/v1/users", json={"username": "user1", "email": "test@example.com"})
    response = client.post("/api/v1/users", json={"username": "user2", "email": "test@example.com"})
    assert response.status_code == 400
    assert "Email already exists" in response.json()["detail"]


def test_delete_user():
    """Test user deletion"""
    create_response = client.post("/api/v1/users", json={"username": "testuser", "email": "test@example.com"})
    user_id = create_response.json()["id"]

    delete_response = client.delete(f"/api/v1/users/{user_id}")
    assert delete_response.status_code == 204

    get_response = client.get(f"/api/v1/users/{user_id}")
    assert get_response.status_code == 404


def test_delete_nonexistent_user():
    """Test deleting a user that doesn't exist"""
    response = client.delete("/api/v1/users/999")
    assert response.status_code == 404
