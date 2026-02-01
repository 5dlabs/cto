// Test fixture for Nova (Elysia/Node backend) detection
import { Elysia } from "elysia";

const app = new Elysia()
  .get("/", () => "Hello from Nova!")
  .get("/health", () => ({ status: "ok" }))
  .listen(3000);

console.log(`Server running at ${app.server?.hostname}:${app.server?.port}`);
