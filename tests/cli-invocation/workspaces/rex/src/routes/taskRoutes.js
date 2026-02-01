/**
 * Task Routes
 * RESTful API routes for task management
 */

const express = require('express');
const taskController = require('../controllers/taskController');
const { validateTask, validateTaskUpdate, validateId } = require('../middleware/validator');

const router = express.Router();

router
  .route('/')
  .get(taskController.getAllTasks)
  .post(validateTask, taskController.createTask);

router.get('/stats', taskController.getStats);

router
  .route('/:id')
  .get(validateId, taskController.getTaskById)
  .patch(validateId, validateTaskUpdate, taskController.updateTask)
  .delete(validateId, taskController.deleteTask);

module.exports = router;
