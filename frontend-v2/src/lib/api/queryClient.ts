import { QueryClient, DefaultOptions } from '@tanstack/react-query';
import { CACHE_TIME, STALE_TIME } from '../constants/api';
import { APIError } from '../types/api';

// Default query options
const defaultQueryOptions: DefaultOptions = {
  queries: {
    // Global defaults for all queries
    staleTime: STALE_TIME.SHORT, // 30 seconds
    gcTime: CACHE_TIME.MEDIUM, // 15 minutes (formerly cacheTime)
    retry: (failureCount, error) => {
      // Don't retry on authentication errors
      const apiError = error as APIError;
      if (apiError?.error?.status_code === 401 || apiError?.error?.status_code === 403) {
        return false;
      }
      // Retry up to 3 times for other errors
      return failureCount < 3;
    },
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    refetchOnWindowFocus: false,
    refetchOnMount: true,
    refetchOnReconnect: true,
  },
  mutations: {
    // Global defaults for all mutations
    retry: (failureCount, error) => {
      // Don't retry mutations on client errors (4xx)
      const apiError = error as APIError;
      if (apiError?.error?.status_code && apiError.error.status_code >= 400 && apiError.error.status_code < 500) {
        return false;
      }
      // Retry up to 2 times for server errors
      return failureCount < 2;
    },
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
  },
};

// Create query client with custom configuration
export const queryClient = new QueryClient({
  defaultOptions: defaultQueryOptions,
});

// Query client configuration for different data types
export const createQueryOptions = {
  // Fast-changing data (user sessions, real-time data)
  realtime: {
    staleTime: STALE_TIME.IMMEDIATE,
    gcTime: CACHE_TIME.SHORT,
    refetchInterval: 30000, // 30 seconds
  },
  
  // User data (profiles, preferences)
  user: {
    staleTime: STALE_TIME.MEDIUM,
    gcTime: CACHE_TIME.LONG,
  },
  
  // System configuration (roles, services, apps)
  config: {
    staleTime: STALE_TIME.LONG,
    gcTime: CACHE_TIME.VERY_LONG,
  },
  
  // Static data (service types, system info)
  static: {
    staleTime: STALE_TIME.LONG,
    gcTime: CACHE_TIME.VERY_LONG,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
  },
  
  // File listings (can change frequently)
  files: {
    staleTime: STALE_TIME.SHORT,
    gcTime: CACHE_TIME.MEDIUM,
  },
};

// Query invalidation helpers
export const invalidateQueries = {
  // Authentication related
  auth: () => queryClient.invalidateQueries({ queryKey: ['auth'] }),
  userSession: () => queryClient.invalidateQueries({ queryKey: ['user', 'session'] }),
  adminSession: () => queryClient.invalidateQueries({ queryKey: ['admin', 'session'] }),
  
  // Users and roles
  users: () => queryClient.invalidateQueries({ queryKey: ['users'] }),
  user: (id?: string | number) => 
    id ? queryClient.invalidateQueries({ queryKey: ['users', id] }) 
        : queryClient.invalidateQueries({ queryKey: ['users'] }),
  roles: () => queryClient.invalidateQueries({ queryKey: ['roles'] }),
  role: (id?: string | number) => 
    id ? queryClient.invalidateQueries({ queryKey: ['roles', id] }) 
        : queryClient.invalidateQueries({ queryKey: ['roles'] }),
  
  // Applications
  apps: () => queryClient.invalidateQueries({ queryKey: ['apps'] }),
  app: (id?: string | number) => 
    id ? queryClient.invalidateQueries({ queryKey: ['apps', id] }) 
        : queryClient.invalidateQueries({ queryKey: ['apps'] }),
  
  // Services
  services: () => queryClient.invalidateQueries({ queryKey: ['services'] }),
  service: (id?: string | number) => 
    id ? queryClient.invalidateQueries({ queryKey: ['services', id] }) 
        : queryClient.invalidateQueries({ queryKey: ['services'] }),
  
  // Configuration
  config: () => queryClient.invalidateQueries({ queryKey: ['config'] }),
  emailTemplates: () => queryClient.invalidateQueries({ queryKey: ['email-templates'] }),
  lookupKeys: () => queryClient.invalidateQueries({ queryKey: ['lookup-keys'] }),
  
  // Files
  files: (path?: string) => 
    path ? queryClient.invalidateQueries({ queryKey: ['files', path] }) 
          : queryClient.invalidateQueries({ queryKey: ['files'] }),
  
  // Scripts and events
  eventScripts: () => queryClient.invalidateQueries({ queryKey: ['event-scripts'] }),
  scheduler: () => queryClient.invalidateQueries({ queryKey: ['scheduler'] }),
  
  // System
  system: () => queryClient.invalidateQueries({ queryKey: ['system'] }),
  limits: () => queryClient.invalidateQueries({ queryKey: ['limits'] }),
  
  // Clear all cache
  all: () => queryClient.clear(),
};

// Prefetch helpers for better UX
export const prefetchQueries = {
  userProfile: () => queryClient.prefetchQuery({
    queryKey: ['user', 'profile'],
    queryFn: () => {
      // This will be implemented by the auth service
      throw new Error('Auth service not implemented');
    },
    ...createQueryOptions.user,
  }),
  
  systemInfo: () => queryClient.prefetchQuery({
    queryKey: ['system', 'info'],
    queryFn: () => {
      // This will be implemented by the system service
      throw new Error('System service not implemented');
    },
    ...createQueryOptions.static,
  }),
};

// Query client event listeners for debugging and monitoring
if (process.env.NODE_ENV === 'development') {
  queryClient.getQueryCache().subscribe((event) => {
    console.log('[QueryClient] Query event:', event);
  });
  
  queryClient.getMutationCache().subscribe((event) => {
    console.log('[QueryClient] Mutation event:', event);
  });
}

export default queryClient;