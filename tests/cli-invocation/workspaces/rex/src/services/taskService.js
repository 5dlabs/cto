/**
 * Task Service
 * Business logic for task management
 */

const { AppError } = require('../middleware/errorHandler');

class TaskService {
  constructor() {
    this.tasks = new Map();
    this.nextId = 1;
  }

  generateId() {
    return `task-${this.nextId++}`;
  }

  createTask(data) {
    const task = {
      id: this.generateId(),
      title: data.title,
      description: data.description || '',
      priority: data.priority || 'medium',
      status: data.status || 'todo',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    this.tasks.set(task.id, task);
    return task;
  }

  getAllTasks(filters = {}) {
    let tasks = Array.from(this.tasks.values());

    if (filters.status) {
      tasks = tasks.filter(task => task.status === filters.status);
    }

    if (filters.priority) {
      tasks = tasks.filter(task => task.priority === filters.priority);
    }

    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      tasks = tasks.filter(task =>
        task.title.toLowerCase().includes(searchLower) ||
        task.description.toLowerCase().includes(searchLower)
      );
    }

    return tasks;
  }

  getTaskById(id) {
    const task = this.tasks.get(id);
    if (!task) {
      throw new AppError('Task not found', 404);
    }
    return task;
  }

  updateTask(id, data) {
    const task = this.getTaskById(id);

    if (data.title !== undefined) task.title = data.title;
    if (data.description !== undefined) task.description = data.description;
    if (data.priority !== undefined) task.priority = data.priority;
    if (data.status !== undefined) task.status = data.status;

    task.updatedAt = new Date().toISOString();

    this.tasks.set(id, task);
    return task;
  }

  deleteTask(id) {
    const task = this.getTaskById(id);
    this.tasks.delete(id);
    return task;
  }

  getStats() {
    const tasks = Array.from(this.tasks.values());
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
    };
  }
}

module.exports = new TaskService();
