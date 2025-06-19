import { QueryClient } from '@tanstack/react-query'

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes (was cacheTime)
      retry: 3,
      refetchOnWindowFocus: false,
      refetchOnReconnect: true,
    },
    mutations: {
      retry: 1,
    },
  },
})

// Query keys factory for consistent key management
export const queryKeys = {
  // Auth
  auth: {
    user: () => ['auth', 'user'] as const,
    session: () => ['auth', 'session'] as const,
  },
  // Users
  users: {
    all: () => ['users'] as const,
    list: (filters?: Record<string, any>) => ['users', 'list', filters] as const,
    detail: (id: string) => ['users', 'detail', id] as const,
  },
  // Roles
  roles: {
    all: () => ['roles'] as const,
    list: (filters?: Record<string, any>) => ['roles', 'list', filters] as const,
    detail: (id: string) => ['roles', 'detail', id] as const,
  },
  // Services
  services: {
    all: () => ['services'] as const,
    list: (filters?: Record<string, any>) => ['services', 'list', filters] as const,
    detail: (id: string) => ['services', 'detail', id] as const,
    types: () => ['services', 'types'] as const,
  },
  // System
  system: {
    info: () => ['system', 'info'] as const,
    health: () => ['system', 'health'] as const,
    config: () => ['system', 'config'] as const,
  },
} as const