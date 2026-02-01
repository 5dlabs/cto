/**
 * Authentication Middleware
 * JWT-based authentication with role-based access control
 */

const crypto = require('crypto');
const { AppError, asyncHandler } = require('./errorHandler');

/**
 * Simple JWT implementation (in production, use jsonwebtoken library)
 */
class JWT {
  constructor(secret) {
    this.secret = secret;
  }

  /**
   * Create a JWT token
   */
  sign(payload, expiresIn = '24h') {
    const header = {
      alg: 'HS256',
      typ: 'JWT',
    };

    const now = Math.floor(Date.now() / 1000);
    const exp = this.parseExpiry(expiresIn);

    const tokenPayload = {
      ...payload,
      iat: now,
      exp: now + exp,
    };

    const encodedHeader = this.base64UrlEncode(JSON.stringify(header));
    const encodedPayload = this.base64UrlEncode(JSON.stringify(tokenPayload));

    const signature = this.createSignature(encodedHeader, encodedPayload);

    return `${encodedHeader}.${encodedPayload}.${signature}`;
  }

  /**
   * Verify and decode a JWT token
   */
  verify(token) {
    const parts = token.split('.');
    if (parts.length !== 3) {
      throw new AppError('Invalid token format', 401);
    }

    const [encodedHeader, encodedPayload, signature] = parts;

    // Verify signature
    const expectedSignature = this.createSignature(encodedHeader, encodedPayload);
    if (signature !== expectedSignature) {
      throw new AppError('Invalid token signature', 401);
    }

    // Decode payload
    const payload = JSON.parse(this.base64UrlDecode(encodedPayload));

    // Check expiration
    const now = Math.floor(Date.now() / 1000);
    if (payload.exp && payload.exp < now) {
      throw new AppError('Token has expired', 401);
    }

    return payload;
  }

  createSignature(encodedHeader, encodedPayload) {
    const data = `${encodedHeader}.${encodedPayload}`;
    return crypto
      .createHmac('sha256', this.secret)
      .update(data)
      .digest('base64url');
  }

  base64UrlEncode(str) {
    return Buffer.from(str).toString('base64url');
  }

  base64UrlDecode(str) {
    return Buffer.from(str, 'base64url').toString('utf-8');
  }

  parseExpiry(expiresIn) {
    const units = {
      s: 1,
      m: 60,
      h: 3600,
      d: 86400,
    };

    const match = expiresIn.match(/^(\d+)([smhd])$/);
    if (!match) {
      throw new Error('Invalid expiry format');
    }

    const [, value, unit] = match;
    return parseInt(value) * units[unit];
  }
}

/**
 * In-memory user store (in production, use a database)
 */
class UserStore {
  constructor() {
    this.users = new Map();
    this.sessions = new Map();
    this.initializeDefaultUsers();
  }

  initializeDefaultUsers() {
    // Default test users
    this.createUser({
      username: 'admin',
      password: 'admin123',
      email: 'admin@example.com',
      role: 'admin',
    });

    this.createUser({
      username: 'user',
      password: 'user123',
      email: 'user@example.com',
      role: 'user',
    });
  }

  hashPassword(password) {
    return crypto.createHash('sha256').update(password).digest('hex');
  }

  createUser(data) {
    const user = {
      id: `user-${this.users.size + 1}`,
      username: data.username,
      password: this.hashPassword(data.password),
      email: data.email,
      role: data.role || 'user',
      createdAt: new Date().toISOString(),
    };

    this.users.set(user.username, user);
    return this.sanitizeUser(user);
  }

  findByUsername(username) {
    return this.users.get(username);
  }

  verifyPassword(user, password) {
    return user.password === this.hashPassword(password);
  }

  sanitizeUser(user) {
    const { password, ...sanitized } = user;
    return sanitized;
  }

  createSession(userId, token) {
    this.sessions.set(token, {
      userId,
      createdAt: new Date().toISOString(),
    });
  }

  getSession(token) {
    return this.sessions.get(token);
  }

  deleteSession(token) {
    this.sessions.delete(token);
  }
}

// Initialize services
const JWT_SECRET = process.env.JWT_SECRET || 'development-secret-key-change-in-production';
const jwt = new JWT(JWT_SECRET);
const userStore = new UserStore();

/**
 * Authentication middleware
 */
const authenticate = asyncHandler(async (req, res, next) => {
  // Get token from header
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    throw new AppError('No authentication token provided', 401);
  }

  const token = authHeader.split(' ')[1];

  try {
    // Verify token
    const payload = jwt.verify(token);

    // Get user from store
    const user = userStore.findByUsername(payload.username);
    if (!user) {
      throw new AppError('User no longer exists', 401);
    }

    // Attach user to request
    req.user = userStore.sanitizeUser(user);
    req.token = token;

    next();
  } catch (error) {
    if (error instanceof AppError) {
      throw error;
    }
    throw new AppError('Invalid or expired token', 401);
  }
});

/**
 * Authorization middleware - check user roles
 */
const authorize = (...roles) => {
  return (req, res, next) => {
    if (!req.user) {
      throw new AppError('Authentication required', 401);
    }

    if (!roles.includes(req.user.role)) {
      throw new AppError('You do not have permission to perform this action', 403);
    }

    next();
  };
};

/**
 * Optional authentication - attach user if token is valid, but don't require it
 */
const optionalAuth = asyncHandler(async (req, res, next) => {
  const authHeader = req.headers.authorization;

  if (authHeader && authHeader.startsWith('Bearer ')) {
    try {
      const token = authHeader.split(' ')[1];
      const payload = jwt.verify(token);
      const user = userStore.findByUsername(payload.username);

      if (user) {
        req.user = userStore.sanitizeUser(user);
        req.token = token;
      }
    } catch (error) {
      // Ignore errors for optional auth
    }
  }

  next();
});

/**
 * Login handler
 */
const login = asyncHandler(async (req, res) => {
  const { username, password } = req.body;

  if (!username || !password) {
    throw new AppError('Please provide username and password', 400);
  }

  // Find user
  const user = userStore.findByUsername(username);
  if (!user) {
    throw new AppError('Invalid credentials', 401);
  }

  // Verify password
  if (!userStore.verifyPassword(user, password)) {
    throw new AppError('Invalid credentials', 401);
  }

  // Create token
  const token = jwt.sign({
    id: user.id,
    username: user.username,
    role: user.role,
  }, '24h');

  // Create session
  userStore.createSession(user.id, token);

  res.status(200).json({
    status: 'success',
    data: {
      user: userStore.sanitizeUser(user),
      token,
      expiresIn: '24h',
    },
  });
});

/**
 * Logout handler
 */
const logout = asyncHandler(async (req, res) => {
  if (req.token) {
    userStore.deleteSession(req.token);
  }

  res.status(200).json({
    status: 'success',
    message: 'Logged out successfully',
  });
});

/**
 * Get current user
 */
const getCurrentUser = asyncHandler(async (req, res) => {
  res.status(200).json({
    status: 'success',
    data: {
      user: req.user,
    },
  });
});

/**
 * Register new user
 */
const register = asyncHandler(async (req, res) => {
  const { username, password, email } = req.body;

  if (!username || !password || !email) {
    throw new AppError('Please provide username, password, and email', 400);
  }

  // Check if user exists
  if (userStore.findByUsername(username)) {
    throw new AppError('Username already exists', 400);
  }

  // Create user
  const user = userStore.createUser({
    username,
    password,
    email,
    role: 'user',
  });

  // Create token
  const token = jwt.sign({
    id: user.id,
    username: user.username,
    role: user.role,
  }, '24h');

  userStore.createSession(user.id, token);

  res.status(201).json({
    status: 'success',
    data: {
      user,
      token,
      expiresIn: '24h',
    },
  });
});

module.exports = {
  authenticate,
  authorize,
  optionalAuth,
  login,
  logout,
  getCurrentUser,
  register,
  jwt,
  userStore,
};
