import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'jsdom',
    include: ['tests/**/*.test.js'],
    setupFiles: ['tests/setup.js'],
    coverage: {
      provider: 'v8',
      include: ['src/**/*.js'],
      reporter: ['text', 'lcov', 'json-summary'],
      thresholds: {
        statements: 72,
        branches: 58,
        functions: 65,
        lines: 75
      }
    }
  }
});
