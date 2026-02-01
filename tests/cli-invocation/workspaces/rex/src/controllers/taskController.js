/**
 * Task Controller
 * Handles HTTP requests for task management
 */

const taskService = require('../services/taskService');
const { asyncHandler } = require('../middleware/errorHandler');

const createTask = asyncHandler(async (req, res) => {
  const task = taskService.createTask(req.body);

  res.status(201).json({
    status: 'success',
    data: { task },
  });
});

const getAllTasks = asyncHandler(async (req, res) => {
  const filters = {
    status: req.query.status,
    priority: req.query.priority,
    search: req.query.search,
  };

  const tasks = taskService.getAllTasks(filters);

  res.status(200).json({
    status: 'success',
    results: tasks.length,
    data: { tasks },
  });
});

const getTaskById = asyncHandler(async (req, res) => {
  const task = taskService.getTaskById(req.params.id);

  res.status(200).json({
    status: 'success',
    data: { task },
  });
});

const updateTask = asyncHandler(async (req, res) => {
  const task = taskService.updateTask(req.params.id, req.body);

  res.status(200).json({
    status: 'success',
    data: { task },
  });
});

const deleteTask = asyncHandler(async (req, res) => {
  taskService.deleteTask(req.params.id);

  res.status(204).json({
    status: 'success',
    data: null,
  });
});

const getStats = asyncHandler(async (req, res) => {
  const stats = taskService.getStats();

  res.status(200).json({
    status: 'success',
    data: { stats },
  });
});

module.exports = {
  createTask,
  getAllTasks,
  getTaskById,
  updateTask,
  deleteTask,
  getStats,
};
