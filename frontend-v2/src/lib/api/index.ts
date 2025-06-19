// Main API integration exports
export * from './client';
export * from './queryClient';

// Service exports
export * from './services/auth';
export * from './services/users';
export * from './services/services';

// Constants and types
export * from '../constants/api';
export * from '../types/api';
export * from '../validation/schemas';

// WebSocket integration
export * from '../websocket/client';

// Mock service worker
export * from '../mocks/browser';
export * from '../mocks/server';

// Re-export React Query essentials
export { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

// Main initialization function
export async function initializeDreamFactoryAPI(config: {
  baseURL?: string;
  apiKey: string;
  enableMocks?: boolean;
  enableWebSocket?: boolean;
  debug?: boolean;
}) {
  const { 
    baseURL = '/api/v2', 
    apiKey, 
    enableMocks = false, 
    enableWebSocket = true,
    debug = false 
  } = config;

  // Initialize API client
  const { initializeAPIClient } = await import('./client');
  const apiClient = initializeAPIClient({
    baseURL,
    apiKey,
    timeout: 30000,
  });

  // Initialize mock service worker if enabled
  if (enableMocks && process.env.NODE_ENV === 'development') {
    const { startMockWorker } = await import('../mocks/browser');
    await startMockWorker();
  }

  // Initialize WebSocket client if enabled
  if (enableWebSocket) {
    const { initializeWebSocketClient, enableAutoConnect } = await import('../websocket/client');
    initializeWebSocketClient({ debug });
    enableAutoConnect();
  }

  return {
    apiClient,
    initialized: true,
  };
}