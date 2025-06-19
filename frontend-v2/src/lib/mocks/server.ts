import { setupServer } from 'msw/node';
import { handlers } from './handlers';

// Setup MSW server for Node.js (testing)
export const server = setupServer(...handlers);

// Server lifecycle management for tests
export function startMockServer() {
  server.listen({
    onUnhandledRequest: 'warn',
  });
  console.log('[MSW] Mock server started for testing');
}

export function stopMockServer() {
  server.close();
  console.log('[MSW] Mock server stopped');
}

export function resetMockServer() {
  server.resetHandlers(...handlers);
}

export default server;