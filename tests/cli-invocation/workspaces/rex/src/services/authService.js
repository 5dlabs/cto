/**
 * Authentication Service
 * Handles JWT token generation, validation, and user authentication
 */

const jwt = require('jsonwebtoken');
const crypto = require('crypto');
const config = require('../config');
const { AppError } = require('../middleware/errorHandler');

class AuthService {
  /**
   * Generate JWT access token
   */
  generateAccessToken(payload) {
    return jwt.sign(payload, config.jwt.secret, {
      expiresIn: config.jwt.expiresIn,
      issuer: 'rex-api',
      audience: 'rex-api-users',
    });
  }

  /**
   * Generate JWT refresh token
   */
  generateRefreshToken(payload) {
    return jwt.sign(payload, config.jwt.secret, {
      expiresIn: config.jwt.refreshExpiresIn,
      issuer: 'rex-api',
      audience: 'rex-api-users',
    });
  }

  /**
   * Verify JWT token
   */
  verifyToken(token) {
    try {
      return jwt.verify(token, config.jwt.secret, {
        issuer: 'rex-api',
        audience: 'rex-api-users',
      });
    } catch (error) {
      if (error.name === 'TokenExpiredError') {
        throw new AppError('Token expired', 401);
      }
      if (error.name === 'JsonWebTokenError') {
        throw new AppError('Invalid token', 401);
      }
      throw new AppError('Token verification failed', 401);
    }
  }

  /**
   * Generate API key
   */
  generateApiKey() {
    return `rex_${crypto.randomBytes(32).toString('hex')}`;
  }

  /**
   * Hash password using crypto
   */
  hashPassword(password) {
    const salt = crypto.randomBytes(16).toString('hex');
    const hash = crypto
      .pbkdf2Sync(password, salt, 10000, 64, 'sha512')
      .toString('hex');
    return { salt, hash };
  }

  /**
   * Verify password
   */
  verifyPassword(password, salt, hash) {
    const verifyHash = crypto
      .pbkdf2Sync(password, salt, 10000, 64, 'sha512')
      .toString('hex');
    return hash === verifyHash;
  }

  /**
   * Login with credentials
   * (Mock implementation - would typically query database)
   */
  async login(username, password) {
    // In production, this would verify against database
    // This is a mock implementation for demonstration
    if (!username || !password) {
      throw new AppError('Username and password are required', 400);
    }

    // Mock user validation
    if (username === 'demo' && password === 'demo123') {
      const user = {
        id: 'user-1',
        username: 'demo',
        email: 'demo@example.com',
        role: 'user',
      };

      const accessToken = this.generateAccessToken({
        userId: user.id,
        username: user.username,
        role: user.role,
      });

      const refreshToken = this.generateRefreshToken({
        userId: user.id,
      });

      return {
        user,
        accessToken,
        refreshToken,
        expiresIn: config.jwt.expiresIn,
      };
    }

    throw new AppError('Invalid credentials', 401);
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshAccessToken(refreshToken) {
    const payload = this.verifyToken(refreshToken);

    if (!payload.userId) {
      throw new AppError('Invalid refresh token', 401);
    }

    // Generate new access token
    const accessToken = this.generateAccessToken({
      userId: payload.userId,
    });

    return {
      accessToken,
      expiresIn: config.jwt.expiresIn,
    };
  }

  /**
   * Logout (invalidate tokens)
   * In production, this would add token to blacklist
   */
  async logout(token) {
    // In production, add token to Redis blacklist with TTL
    // For now, just verify token is valid
    this.verifyToken(token);
    return { message: 'Logged out successfully' };
  }

  /**
   * Get user info from token
   */
  getUserFromToken(token) {
    const payload = this.verifyToken(token);
    return {
      userId: payload.userId,
      username: payload.username,
      role: payload.role,
    };
  }
}

module.exports = new AuthService();
