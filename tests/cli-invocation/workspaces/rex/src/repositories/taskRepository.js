/**
 * Task Repository
 * Data access layer with database abstraction
 * Demonstrates repository pattern for clean architecture
 */

const { AppError } = require('../middleware/errorHandler');

/**
 * Base Repository providing common database operations
 */
class BaseRepository {
  constructor(dataStore) {
    this.store = dataStore;
  }

  async findAll(filters = {}) {
    throw new Error('findAll must be implemented');
  }

  async findById(id) {
    throw new Error('findById must be implemented');
  }

  async create(data) {
    throw new Error('create must be implemented');
  }

  async update(id, data) {
    throw new Error('update must be implemented');
  }

  async delete(id) {
    throw new Error('delete must be implemented');
  }
}

/**
 * Task Repository Implementation
 * In production, this would connect to a real database (PostgreSQL, MongoDB, etc.)
 */
class TaskRepository extends BaseRepository {
  constructor() {
    super(new Map());
    this.nextId = 1;
    this.indexes = {
      status: new Map(),
      priority: new Map(),
    };
  }

  /**
   * Generate unique task ID
   */
  generateId() {
    return `task-${this.nextId++}`;
  }

  /**
   * Update indexes for efficient querying
   */
  updateIndexes(task, oldTask = null) {
    // Remove old indexes
    if (oldTask) {
      this._removeFromIndex('status', oldTask.status, oldTask.id);
      this._removeFromIndex('priority', oldTask.priority, oldTask.id);
    }

    // Add new indexes
    this._addToIndex('status', task.status, task.id);
    this._addToIndex('priority', task.priority, task.id);
  }

  _addToIndex(indexName, key, taskId) {
    if (!this.indexes[indexName].has(key)) {
      this.indexes[indexName].set(key, new Set());
    }
    this.indexes[indexName].get(key).add(taskId);
  }

  _removeFromIndex(indexName, key, taskId) {
    if (this.indexes[indexName].has(key)) {
      this.indexes[indexName].get(key).delete(taskId);
    }
  }

  /**
   * Create a new task
   */
  async create(data) {
    const task = {
      id: this.generateId(),
      title: data.title,
      description: data.description || '',
      priority: data.priority || 'medium',
      status: data.status || 'todo',
      tags: data.tags || [],
      assignee: data.assignee || null,
      dueDate: data.dueDate || null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      version: 1,
    };

    this.store.set(task.id, task);
    this.updateIndexes(task);

    return task;
  }

  /**
   * Find all tasks with optional filters
   */
  async findAll(filters = {}, options = {}) {
    let taskIds;

    // Use indexes for efficient filtering
    if (filters.status && this.indexes.status.has(filters.status)) {
      taskIds = Array.from(this.indexes.status.get(filters.status));
    } else if (filters.priority && this.indexes.priority.has(filters.priority)) {
      taskIds = Array.from(this.indexes.priority.get(filters.priority));
    } else {
      taskIds = Array.from(this.store.keys());
    }

    // Get tasks from store
    let tasks = taskIds.map(id => this.store.get(id)).filter(Boolean);

    // Apply additional filters
    if (filters.status && !this.indexes.status.has(filters.status)) {
      tasks = tasks.filter(task => task.status === filters.status);
    }

    if (filters.priority && !this.indexes.priority.has(filters.priority)) {
      tasks = tasks.filter(task => task.priority === filters.priority);
    }

    if (filters.assignee) {
      tasks = tasks.filter(task => task.assignee === filters.assignee);
    }

    if (filters.tags && filters.tags.length > 0) {
      tasks = tasks.filter(task =>
        filters.tags.some(tag => task.tags.includes(tag))
      );
    }

    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      tasks = tasks.filter(task =>
        task.title.toLowerCase().includes(searchLower) ||
        task.description.toLowerCase().includes(searchLower)
      );
    }

    // Apply sorting
    if (options.sortBy) {
      const [field, order] = options.sortBy.split(':');
      tasks.sort((a, b) => {
        const aVal = a[field];
        const bVal = b[field];
        const comparison = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
        return order === 'desc' ? -comparison : comparison;
      });
    }

    // Apply pagination
    const page = options.page || 1;
    const limit = options.limit || 100;
    const startIndex = (page - 1) * limit;
    const endIndex = page * limit;

    const paginatedTasks = tasks.slice(startIndex, endIndex);

    return {
      tasks: paginatedTasks,
      pagination: {
        total: tasks.length,
        page,
        limit,
        pages: Math.ceil(tasks.length / limit),
      },
    };
  }

  /**
   * Find a task by ID
   */
  async findById(id) {
    const task = this.store.get(id);
    if (!task) {
      throw new AppError('Task not found', 404);
    }
    return task;
  }

  /**
   * Update a task with optimistic locking
   */
  async update(id, data, version = null) {
    const task = await this.findById(id);

    // Optimistic locking check
    if (version !== null && task.version !== version) {
      throw new AppError(
        'Task was modified by another request. Please refresh and try again.',
        409
      );
    }

    const oldTask = { ...task };

    // Update fields
    if (data.title !== undefined) task.title = data.title;
    if (data.description !== undefined) task.description = data.description;
    if (data.priority !== undefined) task.priority = data.priority;
    if (data.status !== undefined) task.status = data.status;
    if (data.tags !== undefined) task.tags = data.tags;
    if (data.assignee !== undefined) task.assignee = data.assignee;
    if (data.dueDate !== undefined) task.dueDate = data.dueDate;

    task.updatedAt = new Date().toISOString();
    task.version += 1;

    this.store.set(id, task);
    this.updateIndexes(task, oldTask);

    return task;
  }

  /**
   * Delete a task
   */
  async delete(id) {
    const task = await this.findById(id);
    this.store.delete(id);

    // Clean up indexes
    this._removeFromIndex('status', task.status, task.id);
    this._removeFromIndex('priority', task.priority, task.id);

    return task;
  }

  /**
   * Bulk operations for efficiency
   */
  async bulkCreate(tasksData) {
    const tasks = await Promise.all(
      tasksData.map(data => this.create(data))
    );
    return tasks;
  }

  async bulkUpdate(updates) {
    const tasks = await Promise.all(
      updates.map(({ id, data, version }) => this.update(id, data, version))
    );
    return tasks;
  }

  async bulkDelete(ids) {
    const tasks = await Promise.all(
      ids.map(id => this.delete(id))
    );
    return tasks;
  }

  /**
   * Get statistics
   */
  async getStats() {
    const tasks = Array.from(this.store.values());

    return {
      total: tasks.length,
      byStatus: {
        todo: tasks.filter(t => t.status === 'todo').length,
        in_progress: tasks.filter(t => t.status === 'in_progress').length,
        completed: tasks.filter(t => t.status === 'completed').length,
        archived: tasks.filter(t => t.status === 'archived').length,
      },
      byPriority: {
        low: tasks.filter(t => t.priority === 'low').length,
        medium: tasks.filter(t => t.priority === 'medium').length,
        high: tasks.filter(t => t.priority === 'high').length,
        urgent: tasks.filter(t => t.priority === 'urgent').length,
      },
      overdue: tasks.filter(t =>
        t.dueDate && new Date(t.dueDate) < new Date() && t.status !== 'completed'
      ).length,
    };
  }

  /**
   * Advanced queries
   */
  async findOverdueTasks() {
    const now = new Date();
    const tasks = Array.from(this.store.values());
    return tasks.filter(t =>
      t.dueDate &&
      new Date(t.dueDate) < now &&
      t.status !== 'completed'
    );
  }

  async findTasksByAssignee(assignee) {
    const tasks = Array.from(this.store.values());
    return tasks.filter(t => t.assignee === assignee);
  }

  async findTasksByTag(tag) {
    const tasks = Array.from(this.store.values());
    return tasks.filter(t => t.tags.includes(tag));
  }
}

module.exports = new TaskRepository();
