/**
 * Server Entry Point
 * Starts the Express server
 */

const app = require('./app');
const config = require('./config');

const PORT = config.server.port;

const server = app.listen(PORT, () => {
  console.log(`
╔═══════════════════════════════════════════════════╗
║   Rex Backend API Server                         ║
║   Environment: ${config.server.env.padEnd(33)}║
║   Port: ${PORT.toString().padEnd(39)}║
║   API Version: ${config.api.version.padEnd(32)}║
╚═══════════════════════════════════════════════════╝

Server is running at http://localhost:${PORT}
API endpoints available at http://localhost:${PORT}${config.api.prefix}

Health check: http://localhost:${PORT}/health
  `);
});

// Graceful shutdown
const shutdown = () => {
  console.log('\nReceived shutdown signal, closing server gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });

  setTimeout(() => {
    console.error('Could not close connections in time, forcefully shutting down');
    process.exit(1);
  }, 10000);
};

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);

// Unhandled rejection handler
process.on('unhandledRejection', (err) => {
  console.error('UNHANDLED REJECTION! Shutting down...');
  console.error(err);
  shutdown();
});

module.exports = server;
