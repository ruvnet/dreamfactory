import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { server } from '../../mocks/server';
import {
  useUsers,
  useUser,
  useCreateUser,
  useUpdateUser,
  useDeleteUser,
  useUserProfile,
  useUpdateUserProfile,
  UsersService,
} from '../services/users';
import { initializeAPIClient } from '../client';

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

describe('UsersService', () => {
  let usersService: UsersService;

  beforeEach(() => {
    usersService = new UsersService();
  });

  describe('getUsers', () => {
    it('should fetch users successfully', async () => {
      const result = await usersService.getUsers();

      expect(result).toHaveProperty('resource');
      expect(result).toHaveProperty('meta');
      expect(Array.isArray(result.resource)).toBe(true);
      expect(result.meta.count).toBeGreaterThanOrEqual(0);
    });

    it('should handle query options', async () => {
      const options = {
        limit: 5,
        offset: 0,
        filter: 'is_active=true',
        sort: 'name',
      };

      const result = await usersService.getUsers(options);

      expect(result).toHaveProperty('resource');
      expect(result.resource.length).toBeLessThanOrEqual(5);
    });
  });

  describe('getUser', () => {
    it('should fetch user by ID successfully', async () => {
      const result = await usersService.getUser(1);

      expect(result).toHaveProperty('id', 1);
      expect(result).toHaveProperty('email');
      expect(result).toHaveProperty('name');
    });

    it('should throw error for non-existent user', async () => {
      await expect(usersService.getUser(999)).rejects.toMatchObject({
        error: {
          status_code: 404,
        },
      });
    });
  });

  describe('createUser', () => {
    it('should create user successfully', async () => {
      const userData = {
        name: 'Test User',
        first_name: 'Test',
        last_name: 'User',
        email: 'test@example.com',
        is_active: true,
      };

      const result = await usersService.createUser(userData);

      expect(result).toHaveProperty('id');
      expect(result.email).toBe(userData.email);
      expect(result.name).toBe(userData.name);
    });

    it('should validate user data', async () => {
      const invalidData = {
        name: '',
        first_name: '',
        last_name: '',
        email: 'invalid-email',
      };

      await expect(usersService.createUser(invalidData)).rejects.toThrow();
    });
  });

  describe('updateUser', () => {
    it('should update user successfully', async () => {
      const updateData = {
        name: 'Updated Name',
        email: 'updated@example.com',
      };

      const result = await usersService.updateUser(1, updateData);

      expect(result).toHaveProperty('id', 1);
      expect(result.name).toBe(updateData.name);
      expect(result.email).toBe(updateData.email);
    });

    it('should throw error for non-existent user', async () => {
      await expect(
        usersService.updateUser(999, { name: 'Test' })
      ).rejects.toMatchObject({
        error: {
          status_code: 404,
        },
      });
    });
  });

  describe('deleteUser', () => {
    it('should delete user successfully', async () => {
      await expect(usersService.deleteUser(1)).resolves.not.toThrow();
    });

    it('should delete multiple users', async () => {
      await expect(usersService.deleteUser([1, 2])).resolves.not.toThrow();
    });

    it('should throw error for non-existent user', async () => {
      await expect(usersService.deleteUser(999)).rejects.toMatchObject({
        error: {
          status_code: 404,
        },
      });
    });
  });

  describe('getUserProfile', () => {
    it('should fetch user profile successfully', async () => {
      const result = await usersService.getUserProfile();

      expect(result).toHaveProperty('id');
      expect(result).toHaveProperty('email');
      expect(result).toHaveProperty('name');
    });
  });

  describe('updateUserProfile', () => {
    it('should update user profile successfully', async () => {
      const updateData = {
        name: 'Updated Profile Name',
        phone: '+1234567890',
      };

      const result = await usersService.updateUserProfile(updateData);

      expect(result.name).toBe(updateData.name);
      expect(result.phone).toBe(updateData.phone);
    });
  });

  describe('changePassword', () => {
    it('should change password successfully', async () => {
      await expect(
        usersService.changePassword('oldpassword', 'newpassword')
      ).resolves.not.toThrow();
    });
  });

  describe('bulk operations', () => {
    it('should bulk create users', async () => {
      const users = [
        {
          name: 'User 1',
          first_name: 'User',
          last_name: '1',
          email: 'user1@example.com',
          is_active: true,
        },
        {
          name: 'User 2',
          first_name: 'User',
          last_name: '2',
          email: 'user2@example.com',
          is_active: true,
        },
      ];

      const result = await usersService.bulkCreateUsers(users);

      expect(result).toHaveProperty('resource');
      expect(Array.isArray(result.resource)).toBe(true);
    });

    it('should bulk update users', async () => {
      const users = [
        {
          id: 1,
          name: 'Updated User 1',
        },
        {
          id: 2,
          name: 'Updated User 2',
        },
      ];

      const result = await usersService.bulkUpdateUsers(users);

      expect(result).toHaveProperty('resource');
      expect(Array.isArray(result.resource)).toBe(true);
    });
  });
});

describe('Users Hooks', () => {
  describe('useUsers', () => {
    it('should fetch users successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useUsers(), { wrapper });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toHaveProperty('resource');
      expect(Array.isArray(result.current.data?.resource)).toBe(true);
    });

    it('should handle query options', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(
        () => useUsers({ limit: 5, filter: 'is_active=true' }),
        { wrapper }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.resource.length).toBeLessThanOrEqual(5);
    });
  });

  describe('useUser', () => {
    it('should fetch user by ID', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useUser(1), { wrapper });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toHaveProperty('id', 1);
    });

    it('should not fetch when ID is falsy', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useUser(0), { wrapper });

      expect(result.current.data).toBeUndefined();
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('useUserProfile', () => {
    it('should fetch user profile', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useUserProfile(), { wrapper });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toHaveProperty('id');
      expect(result.current.data).toHaveProperty('email');
    });
  });

  describe('useCreateUser', () => {
    it('should create user successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useCreateUser(), { wrapper });

      const userData = {
        name: 'New User',
        first_name: 'New',
        last_name: 'User',
        email: 'new@example.com',
        is_active: true,
      };

      result.current.mutate(userData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toHaveProperty('id');
      expect(result.current.data?.email).toBe(userData.email);
    });

    it('should handle validation errors', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useCreateUser(), { wrapper });

      const invalidData = {
        name: '',
        first_name: '',
        last_name: '',
        email: 'invalid-email',
      };

      result.current.mutate(invalidData);

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });
    });
  });

  describe('useUpdateUser', () => {
    it('should update user successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useUpdateUser(), { wrapper });

      const updateData = {
        id: 1,
        data: {
          name: 'Updated Name',
        },
      };

      result.current.mutate(updateData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.name).toBe(updateData.data.name);
    });
  });

  describe('useUpdateUserProfile', () => {
    it('should update user profile successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useUpdateUserProfile(), { wrapper });

      const updateData = {
        name: 'Updated Profile Name',
        phone: '+1234567890',
      };

      result.current.mutate(updateData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.name).toBe(updateData.name);
    });
  });

  describe('useDeleteUser', () => {
    it('should delete user successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useDeleteUser(), { wrapper });

      result.current.mutate(1);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });
    });

    it('should delete multiple users', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useDeleteUser(), { wrapper });

      result.current.mutate([1, 2]);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });
    });
  });

  describe('useChangePassword', () => {
    it('should change password successfully', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useChangePassword(), { wrapper });

      const passwordData = {
        currentPassword: 'oldpassword',
        newPassword: 'newpassword',
      };

      result.current.mutate(passwordData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });
    });
  });

  describe('bulk operations hooks', () => {
    it('should bulk create users', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useBulkCreateUsers(), { wrapper });

      const users = [
        {
          name: 'Bulk User 1',
          first_name: 'Bulk',
          last_name: 'User 1',
          email: 'bulk1@example.com',
          is_active: true,
        },
      ];

      result.current.mutate(users);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toHaveProperty('resource');
    });

    it('should bulk update users', async () => {
      const wrapper = createWrapper();
      const { result } = renderHook(() => useBulkUpdateUsers(), { wrapper });

      const users = [
        {
          id: 1,
          name: 'Bulk Updated User',
        },
      ];

      result.current.mutate(users);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toHaveProperty('resource');
    });
  });
});