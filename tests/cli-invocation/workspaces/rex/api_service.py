"""
REX Agent Backend API Demonstration
A simple REST API service demonstrating backend development best practices
"""

from fastapi import FastAPI, HTTPException, status
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import datetime
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="REX Backend API Demo",
    description="Demonstration of backend development capabilities",
    version="1.0.0"
)


class User(BaseModel):
    id: Optional[int] = None
    username: str = Field(..., min_length=3, max_length=50)
    email: str = Field(..., regex=r'^[\w\.-]+@[\w\.-]+\.\w+$')
    created_at: Optional[datetime] = None


class UserCreate(BaseModel):
    username: str = Field(..., min_length=3, max_length=50)
    email: str = Field(..., regex=r'^[\w\.-]+@[\w\.-]+\.\w+$')


in_memory_db: List[User] = []
user_id_counter = 1


@app.get("/health")
async def health_check():
    """Health check endpoint for monitoring"""
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat(),
        "service": "rex-api-demo"
    }


@app.get("/metrics")
async def metrics():
    """Metrics endpoint for observability"""
    return {
        "total_users": len(in_memory_db),
        "timestamp": datetime.utcnow().isoformat(),
        "uptime": "available"
    }


@app.get("/api/v1/users", response_model=List[User])
async def get_users():
    """
    Retrieve all users

    Returns:
        List of all users in the system
    """
    logger.info(f"GET /api/v1/users - Retrieving {len(in_memory_db)} users")
    return in_memory_db


@app.get("/api/v1/users/{user_id}", response_model=User)
async def get_user(user_id: int):
    """
    Retrieve a specific user by ID

    Args:
        user_id: The unique identifier of the user

    Returns:
        User object if found

    Raises:
        HTTPException: 404 if user not found
    """
    logger.info(f"GET /api/v1/users/{user_id}")

    user = next((u for u in in_memory_db if u.id == user_id), None)
    if not user:
        logger.warning(f"User {user_id} not found")
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"User with id {user_id} not found"
        )

    return user


@app.post("/api/v1/users", response_model=User, status_code=status.HTTP_201_CREATED)
async def create_user(user_data: UserCreate):
    """
    Create a new user

    Args:
        user_data: User creation data (username and email)

    Returns:
        Created user object with assigned ID

    Raises:
        HTTPException: 400 if username or email already exists
    """
    global user_id_counter

    logger.info(f"POST /api/v1/users - Creating user: {user_data.username}")

    if any(u.username == user_data.username for u in in_memory_db):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Username already exists"
        )

    if any(u.email == user_data.email for u in in_memory_db):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Email already exists"
        )

    new_user = User(
        id=user_id_counter,
        username=user_data.username,
        email=user_data.email,
        created_at=datetime.utcnow()
    )

    in_memory_db.append(new_user)
    user_id_counter += 1

    logger.info(f"User created successfully: {new_user.id}")
    return new_user


@app.delete("/api/v1/users/{user_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_user(user_id: int):
    """
    Delete a user by ID

    Args:
        user_id: The unique identifier of the user to delete

    Raises:
        HTTPException: 404 if user not found
    """
    global in_memory_db

    logger.info(f"DELETE /api/v1/users/{user_id}")

    user = next((u for u in in_memory_db if u.id == user_id), None)
    if not user:
        logger.warning(f"User {user_id} not found for deletion")
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"User with id {user_id} not found"
        )

    in_memory_db = [u for u in in_memory_db if u.id != user_id]
    logger.info(f"User {user_id} deleted successfully")


@app.exception_handler(Exception)
async def global_exception_handler(request, exc):
    """Global exception handler for unhandled errors"""
    logger.error(f"Unhandled exception: {exc}", exc_info=True)
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={
            "detail": "Internal server error",
            "timestamp": datetime.utcnow().isoformat()
        }
    )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
