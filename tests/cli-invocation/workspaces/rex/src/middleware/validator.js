/**
 * Request Validation Middleware
 * Input validation and sanitization
 */

const { AppError } = require('./errorHandler');

const validateTask = (req, res, next) => {
  const { title, description, priority, status } = req.body;

  const errors = [];

  if (!title || typeof title !== 'string' || title.trim().length === 0) {
    errors.push('Title is required and must be a non-empty string');
  }

  if (title && title.length > 200) {
    errors.push('Title must not exceed 200 characters');
  }

  if (description && typeof description !== 'string') {
    errors.push('Description must be a string');
  }

  if (description && description.length > 2000) {
    errors.push('Description must not exceed 2000 characters');
  }

  if (priority && !['low', 'medium', 'high', 'urgent'].includes(priority)) {
    errors.push('Priority must be one of: low, medium, high, urgent');
  }

  if (status && !['todo', 'in_progress', 'completed', 'archived'].includes(status)) {
    errors.push('Status must be one of: todo, in_progress, completed, archived');
  }

  if (errors.length > 0) {
    return next(new AppError(errors.join('; '), 400));
  }

  req.body.title = title.trim();
  if (description) {
    req.body.description = description.trim();
  }

  next();
};

const validateTaskUpdate = (req, res, next) => {
  const { title, description, priority, status } = req.body;
  const errors = [];

  if (!title && !description && !priority && !status) {
    return next(new AppError('At least one field must be provided for update', 400));
  }

  if (title !== undefined) {
    if (typeof title !== 'string' || title.trim().length === 0) {
      errors.push('Title must be a non-empty string');
    }
    if (title.length > 200) {
      errors.push('Title must not exceed 200 characters');
    }
  }

  if (description !== undefined) {
    if (typeof description !== 'string') {
      errors.push('Description must be a string');
    }
    if (description.length > 2000) {
      errors.push('Description must not exceed 2000 characters');
    }
  }

  if (priority && !['low', 'medium', 'high', 'urgent'].includes(priority)) {
    errors.push('Priority must be one of: low, medium, high, urgent');
  }

  if (status && !['todo', 'in_progress', 'completed', 'archived'].includes(status)) {
    errors.push('Status must be one of: todo, in_progress, completed, archived');
  }

  if (errors.length > 0) {
    return next(new AppError(errors.join('; '), 400));
  }

  if (title) req.body.title = title.trim();
  if (description) req.body.description = description.trim();

  next();
};

const validateId = (req, res, next) => {
  const { id } = req.params;

  if (!id || !/^[a-zA-Z0-9-_]+$/.test(id)) {
    return next(new AppError('Invalid ID format', 400));
  }

  next();
};

module.exports = {
  validateTask,
  validateTaskUpdate,
  validateId,
};
