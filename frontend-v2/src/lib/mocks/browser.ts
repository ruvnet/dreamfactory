import { setupWorker } from 'msw/browser';
import { handlers } from './handlers';

// Setup MSW worker for browser
export const worker = setupWorker(...handlers);

// Start the worker
export async function startMockWorker() {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    await worker.start({
      onUnhandledRequest: 'warn',
      serviceWorker: {
        url: '/mockServiceWorker.js',
      },
    });
    console.log('[MSW] Mock service worker started successfully');
  } catch (error) {
    console.error('[MSW] Failed to start mock service worker:', error);
  }
}

// Stop the worker
export async function stopMockWorker() {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    await worker.stop();
    console.log('[MSW] Mock service worker stopped');
  } catch (error) {
    console.error('[MSW] Failed to stop mock service worker:', error);
  }
}

// Reset handlers
export function resetMockHandlers() {
  worker.resetHandlers(...handlers);
}

// Add custom handlers at runtime
export function addMockHandlers(...newHandlers: any[]) {
  worker.use(...newHandlers);
}

export default worker;