import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { server } from '../../mocks/server';
import {
  useLogin,
  useSession,
  useLogout,
  useRegister,
  AuthService,
} from '../services/auth';
import { initializeAPIClient } from '../client';

// Mock API client
beforeEach(() => {
  server.listen();
  initializeAPIClient({
    baseURL: 'http://localhost:3000/api/v2',
    apiKey: 'test-api-key',
  });
});

afterEach(() => {
  server.resetHandlers();
});

afterEach(() => {
  server.close();
});

// Test wrapper with QueryClient
const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
      mutations: {
        retry: false,
      },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    React.createElement(QueryClientProvider, { client: queryClient }, children)
  );
};

describe('AuthService', () => {
  let authService: AuthService;

  beforeEach(() => {
    authService = new AuthService();
  });

  describe('login', () => {
    it('should login successfully with valid credentials', async () => {
      const credentials = {
        email: 'john.doe@example.com',
        password: 'password',
      };

      const result = await authService.login(credentials);

      expect(result).toMatchObject({
        id: 1,
        email: 'john.doe@example.com',
        session_token: 'mock-session-token-123',
      });
    });

    it('should throw error with invalid credentials', async () => {
      const credentials = {
        email: 'invalid@example.com',
        password: 'wrongpassword',
      };

      await expect(authService.login(credentials)).rejects.toMatchObject({
        error: {
          status_code: 401,
          message: 'Invalid credentials',
        },
      });
    });

    it('should validate input data', async () => {
      const invalidCredentials = {
        email: 'invalid-email',
        password: '',
      };

      await expect(authService.login(invalidCredentials)).rejects.toThrow();
    });
  });

  describe('loginWithToken', () => {
    it('should login with valid JWT token', async () => {
      const result = await authService.loginWithToken('valid-jwt-token');

      expect(result).toMatchObject({
        id: 1,
        email: 'john.doe@example.com',
      });
    });
  });

  describe('register', () => {
    it('should register new user successfully', async () => {
      const registerData = {
        name: 'Test User',
        first_name: 'Test',
        last_name: 'User',
        email: 'test@example.com',
        password: 'password123',
      };

      const result = await authService.register(registerData);

      expect(result).toMatchObject({
        success: true,
      });
    });

    it('should validate registration data', async () => {
      const invalidData = {
        name: '',
        first_name: '',
        last_name: '',
        email: 'invalid-email',
        password: '123', // Too short
      };

      await expect(authService.register(invalidData)).rejects.toThrow();
    });
  });

  describe('checkSession', () => {
    it('should return null when no token', async () => {
      const result = await authService.checkSession();
      expect(result).toBeNull();
    });

    it('should return user data when valid token exists', async () => {
      // Mock setting a token
      const apiClient = authService['client'];
      apiClient.setSessionToken('valid-token');

      const result = await authService.checkSession();

      expect(result).toMatchObject({
        id: 1,
        email: 'john.doe@example.com',
      });
    });
  });

  describe('logout', () => {
    it('should logout successfully', async () => {
      await expect(authService.logout()).resolves.not.toThrow();
    });
  });
});

describe('Auth Hooks', () => {
  describe('useLogin', () => {
    it('should login successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useLogin(), { wrapper });

      const credentials = {
        email: 'john.doe@example.com',
        password: 'password',
      };

      result.current.mutate(credentials);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toMatchObject({
        id: 1,
        email: 'john.doe@example.com',
      });
    });

    it('should handle login error', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useLogin(), { wrapper });

      const credentials = {
        email: 'invalid@example.com',
        password: 'wrongpassword',
      };

      result.current.mutate(credentials);

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error).toMatchObject({
        error: {
          status_code: 401,
        },
      });
    });
  });

  describe('useSession', () => {
    it('should return session data', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useSession(), { wrapper });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toMatchObject({
        id: 1,
        email: 'john.doe@example.com',
      });
    });
  });

  describe('useLogout', () => {
    it('should logout successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useLogout(), { wrapper });

      result.current.mutate({ isSysAdmin: false });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });
    });
  });

  describe('useRegister', () => {
    it('should register successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useRegister(), { wrapper });

      const registerData = {
        name: 'Test User',
        first_name: 'Test',
        last_name: 'User',
        email: 'test@example.com',
        password: 'password123',
      };

      result.current.mutate(registerData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toMatchObject({
        success: true,
      });
    });
  });
});

describe('Auth State Helpers', () => {
  it('should return authentication status', async () => {
    const wrapper = createWrapper();
    const { result } = renderHook(() => {
      const { useIsAuthenticated } = require('../services/auth');
      return useIsAuthenticated();
    }, { wrapper });

    await waitFor(() => {
      expect(typeof result.current).toBe('boolean');
    });
  });

  it('should return current user', async () => {
    const wrapper = createWrapper();
    const { result } = renderHook(() => {
      const { useCurrentUser } = require('../services/auth');
      return useCurrentUser();
    }, { wrapper });

    await waitFor(() => {
      expect(result.current.user).toBeDefined();
    });
  });

  it('should return admin status', async () => {
    const wrapper = createWrapper();
    const { result } = renderHook(() => {
      const { useIsAdmin } = require('../services/auth');
      return useIsAdmin();
    }, { wrapper });

    await waitFor(() => {
      expect(typeof result.current).toBe('boolean');
    });
  });
});