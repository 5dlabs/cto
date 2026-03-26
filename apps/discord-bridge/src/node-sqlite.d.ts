declare module "node:sqlite" {
  export class DatabaseSync {
    constructor(path: string);
    exec(sql: string): void;
    close(): void;
    prepare(sql: string): {
      run: (...args: unknown[]) => unknown;
      get: (...args: unknown[]) => unknown;
      all: (...args: unknown[]) => unknown;
    };
    transaction<T extends (...args: never[]) => unknown>(fn: T): T;
  }
}
