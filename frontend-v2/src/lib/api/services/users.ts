import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { getAPIClient } from '../client';
import { API_ENDPOINTS, QUERY_KEYS } from '../../constants/api';
import { createQueryOptions, invalidateQueries } from '../queryClient';
import {
  User,
  APIListResponse,
  RequestOptions,
  UseMutationOptions,
  UseQueryOptions,
  ID,
} from '../../types/api';
import { UserSchema, UserListResponseSchema } from '../../validation/schemas';

// Users Service Class
export class UsersService {
  private client = getAPIClient();
  
  // Get all users
  async getUsers(options?: RequestOptions): Promise<APIListResponse<User>> {
    const response = await this.client.getAll<APIListResponse<User>>(
      API_ENDPOINTS.SYSTEM_USER,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserListResponseSchema.parse(response);
  }
  
  // Get user by ID
  async getUser(id: ID, options?: RequestOptions): Promise<User> {
    const response = await this.client.getById<User>(
      API_ENDPOINTS.SYSTEM_USER,
      id,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserSchema.parse(response);
  }
  
  // Create new user
  async createUser(data: Partial<User>, options?: RequestOptions): Promise<User> {
    const validatedData = UserSchema.omit({ 
      id: true, 
      created_date: true, 
      last_modified_date: true 
    }).parse(data);
    
    const response = await this.client.create<User>(
      API_ENDPOINTS.SYSTEM_USER,
      validatedData,
      {
        snackbarSuccess: 'User created successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserSchema.parse(response);
  }
  
  // Update user
  async updateUser(id: ID, data: Partial<User>, options?: RequestOptions): Promise<User> {
    const validatedData = UserSchema.partial().parse(data);
    
    const response = await this.client.update<User>(
      API_ENDPOINTS.SYSTEM_USER,
      id,
      validatedData,
      {
        snackbarSuccess: 'User updated successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserSchema.parse(response);
  }
  
  // Delete user(s)
  async deleteUser(id: ID | ID[], options?: RequestOptions): Promise<void> {
    await this.client.remove(
      API_ENDPOINTS.SYSTEM_USER,
      id,
      {
        snackbarSuccess: Array.isArray(id) ? 'Users deleted successfully' : 'User deleted successfully',
        snackbarError: 'server',
        ...options,
      }
    );
  }
  
  // Get user profile (current user)
  async getUserProfile(options?: RequestOptions): Promise<User> {
    const response = await this.client.get<User>(
      API_ENDPOINTS.USER_PROFILE,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserSchema.parse(response);
  }
  
  // Update user profile (current user)
  async updateUserProfile(data: Partial<User>, options?: RequestOptions): Promise<User> {
    const validatedData = UserSchema.partial().parse(data);
    
    const response = await this.client.put<User>(
      API_ENDPOINTS.USER_PROFILE,
      validatedData,
      {
        snackbarSuccess: 'Profile updated successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserSchema.parse(response);
  }
  
  // Change user password
  async changePassword(
    currentPassword: string,
    newPassword: string,
    options?: RequestOptions
  ): Promise<void> {
    await this.client.post(
      API_ENDPOINTS.USER_PASSWORD,
      {
        old_password: currentPassword,
        new_password: newPassword,
      },
      {
        snackbarSuccess: 'Password changed successfully',
        snackbarError: 'server',
        ...options,
      }
    );
  }
  
  // Bulk operations
  async bulkCreateUsers(users: Partial<User>[], options?: RequestOptions): Promise<APIListResponse<User>> {
    const validatedData = users.map(user => 
      UserSchema.omit({ 
        id: true, 
        created_date: true, 
        last_modified_date: true 
      }).parse(user)
    );
    
    const response = await this.client.create<APIListResponse<User>>(
      API_ENDPOINTS.SYSTEM_USER,
      { resource: validatedData },
      {
        snackbarSuccess: 'Users created successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserListResponseSchema.parse(response);
  }
  
  async bulkUpdateUsers(users: Array<Partial<User> & { id: ID }>, options?: RequestOptions): Promise<APIListResponse<User>> {
    const validatedData = users.map(user => UserSchema.partial().parse(user));
    
    const response = await this.client.patch<APIListResponse<User>>(
      API_ENDPOINTS.SYSTEM_USER,
      { resource: validatedData },
      {
        snackbarSuccess: 'Users updated successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return UserListResponseSchema.parse(response);
  }
}

// Create service instance
const usersService = new UsersService();

// React Query Hooks

// Get all users query
export function useUsers(options?: RequestOptions & UseQueryOptions<APIListResponse<User>>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: [...QUERY_KEYS.USERS, requestOptions],
    queryFn: () => usersService.getUsers(requestOptions),
    ...createQueryOptions.user,
    enabled,
    retry,
    staleTime,
    gcTime,
    refetchOnWindowFocus,
    refetchOnMount,
    refetchOnReconnect,
    select,
  });
}

// Get user by ID query
export function useUser(id: ID, options?: RequestOptions & UseQueryOptions<User>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: QUERY_KEYS.USER(id),
    queryFn: () => usersService.getUser(id, requestOptions),
    ...createQueryOptions.user,
    enabled: enabled && !!id,
    retry,
    staleTime,
    gcTime,
    refetchOnWindowFocus,
    refetchOnMount,
    refetchOnReconnect,
    select,
  });
}

// Get user profile query
export function useUserProfile(options?: RequestOptions & UseQueryOptions<User>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: QUERY_KEYS.USER_PROFILE,
    queryFn: () => usersService.getUserProfile(requestOptions),
    ...createQueryOptions.user,
    enabled,
    retry,
    staleTime,
    gcTime,
    refetchOnWindowFocus,
    refetchOnMount,
    refetchOnReconnect,
    select,
  });
}

// Create user mutation
export function useCreateUser(options?: UseMutationOptions<User, any, Partial<User>>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: Partial<User>) => usersService.createUser(data),
    onSuccess: (data, variables) => {
      // Invalidate users list
      invalidateQueries.users();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Update user mutation
export function useUpdateUser(options?: UseMutationOptions<User, any, { id: ID; data: Partial<User> }>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }) => usersService.updateUser(id, data),
    onSuccess: (data, variables) => {
      // Update specific user in cache
      queryClient.setQueryData(QUERY_KEYS.USER(variables.id), data);
      // Invalidate users list
      invalidateQueries.users();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Update user profile mutation
export function useUpdateUserProfile(options?: UseMutationOptions<User, any, Partial<User>>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: Partial<User>) => usersService.updateUserProfile(data),
    onSuccess: (data, variables) => {
      // Update profile in cache
      queryClient.setQueryData(QUERY_KEYS.USER_PROFILE, data);
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Delete user mutation
export function useDeleteUser(options?: UseMutationOptions<void, any, ID | ID[]>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: ID | ID[]) => usersService.deleteUser(id),
    onSuccess: (data, variables) => {
      // Remove from cache and invalidate list
      if (Array.isArray(variables)) {
        variables.forEach(id => {
          queryClient.removeQueries({ queryKey: QUERY_KEYS.USER(id) });
        });
      } else {
        queryClient.removeQueries({ queryKey: QUERY_KEYS.USER(variables) });
      }
      invalidateQueries.users();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Change password mutation
export function useChangePassword(options?: UseMutationOptions<void, any, { currentPassword: string; newPassword: string }>) {
  return useMutation({
    mutationFn: ({ currentPassword, newPassword }) => 
      usersService.changePassword(currentPassword, newPassword),
    onSuccess: options?.onSuccess,
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Bulk create users mutation
export function useBulkCreateUsers(options?: UseMutationOptions<APIListResponse<User>, any, Partial<User>[]>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (users: Partial<User>[]) => usersService.bulkCreateUsers(users),
    onSuccess: (data, variables) => {
      invalidateQueries.users();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Bulk update users mutation
export function useBulkUpdateUsers(options?: UseMutationOptions<APIListResponse<User>, any, Array<Partial<User> & { id: ID }>>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (users: Array<Partial<User> & { id: ID }>) => usersService.bulkUpdateUsers(users),
    onSuccess: (data, variables) => {
      // Update individual users in cache
      variables.forEach(user => {
        const updatedUser = data.resource?.find(u => u.id === user.id);
        if (updatedUser) {
          queryClient.setQueryData(QUERY_KEYS.USER(user.id), updatedUser);
        }
      });
      invalidateQueries.users();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Export service instance
export { usersService };