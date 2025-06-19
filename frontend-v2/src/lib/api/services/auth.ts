import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { getAPIClient } from '../client';
import { API_ENDPOINTS, QUERY_KEYS } from '../../constants/api';
import { createQueryOptions, invalidateQueries } from '../queryClient';
import {
  LoginCredentials,
  RegisterDetails,
  UserSession,
  APIResponse,
  RequestOptions,
  UseMutationOptions,
  UseQueryOptions,
} from '../../types/api';
import {
  LoginCredentialsSchema,
  RegisterDetailsSchema,
  UserSessionSchema,
} from '../../validation/schemas';

// Authentication Service Class
export class AuthService {
  private client = getAPIClient();
  
  // Login with email and password
  async login(credentials: LoginCredentials): Promise<UserSession> {
    const validatedData = LoginCredentialsSchema.parse(credentials);
    
    try {
      // Try user session first
      const response = await this.client.post<UserSession>(
        API_ENDPOINTS.USER_SESSION,
        validatedData,
        { showSpinner: true, snackbarError: 'server' }
      );
      
      const userData = UserSessionSchema.parse(response);
      this.client.setSessionToken(userData.session_token);
      return userData;
    } catch (error) {
      // Fallback to admin session
      try {
        const response = await this.client.post<UserSession>(
          API_ENDPOINTS.ADMIN_SESSION,
          validatedData,
          { showSpinner: true, snackbarError: 'server' }
        );
        
        const userData = UserSessionSchema.parse(response);
        userData.isSysAdmin = true;
        this.client.setSessionToken(userData.session_token);
        return userData;
      } catch (adminError) {
        throw adminError;
      }
    }
  }
  
  // Login with JWT token
  async loginWithToken(jwt?: string): Promise<UserSession> {
    const options: RequestOptions = {
      showSpinner: true,
      snackbarError: 'server',
      additionalHeaders: jwt ? { Authorization: `Bearer ${jwt}` } : undefined,
    };
    
    const response = await this.client.get<UserSession>(
      API_ENDPOINTS.USER_SESSION,
      options
    );
    
    const userData = UserSessionSchema.parse(response);
    this.client.setSessionToken(userData.session_token);
    return userData;
  }
  
  // OAuth login
  async oauthLogin(oauthToken: string, code: string, state: string): Promise<UserSession> {
    const response = await this.client.post<UserSession>(
      API_ENDPOINTS.USER_SESSION,
      null,
      {
        showSpinner: true,
        snackbarError: 'server',
        additionalParams: {
          oauth_callback: true,
          oauth_token: oauthToken,
          code,
          state,
        },
      }
    );
    
    const userData = UserSessionSchema.parse(response);
    this.client.setSessionToken(userData.session_token);
    return userData;
  }
  
  // Register new user
  async register(data: RegisterDetails): Promise<APIResponse> {
    const validatedData = RegisterDetailsSchema.parse(data);
    
    return this.client.post<APIResponse>(
      API_ENDPOINTS.REGISTER,
      validatedData,
      { showSpinner: true, snackbarError: 'server' }
    );
  }
  
  // Check current session
  async checkSession(): Promise<UserSession | null> {
    const token = this.client.getSessionToken();
    if (!token) {
      return null;
    }
    
    try {
      const userData = await this.loginWithToken();
      return userData;
    } catch (error) {
      this.client.setSessionToken(null);
      return null;
    }
  }
  
  // Logout
  async logout(isSysAdmin?: boolean): Promise<void> {
    const endpoint = isSysAdmin ? API_ENDPOINTS.ADMIN_SESSION : API_ENDPOINTS.USER_SESSION;
    
    try {
      await this.client.delete(endpoint);
    } finally {
      this.client.setSessionToken(null);
    }
  }
  
  // Forgot password
  async forgotPassword(email: string): Promise<APIResponse> {
    return this.client.post<APIResponse>(
      API_ENDPOINTS.USER_PASSWORD,
      { email },
      { showSpinner: true, snackbarError: 'server' }
    );
  }
  
  // Reset password
  async resetPassword(token: string, password: string, email?: string): Promise<APIResponse> {
    return this.client.post<APIResponse>(
      API_ENDPOINTS.USER_PASSWORD,
      { token, password, email },
      { showSpinner: true, snackbarError: 'server' }
    );
  }
}

// Create service instance
const authService = new AuthService();

// React Query Hooks

// Login mutation
export function useLogin(options?: UseMutationOptions<UserSession, any, LoginCredentials>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (credentials: LoginCredentials) => authService.login(credentials),
    onSuccess: (userData, variables) => {
      // Update session cache
      queryClient.setQueryData(QUERY_KEYS.USER_SESSION, userData);
      
      // Invalidate user-related queries
      invalidateQueries.auth();
      
      options?.onSuccess?.(userData, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Login with token mutation
export function useLoginWithToken(options?: UseMutationOptions<UserSession, any, string | undefined>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (jwt?: string) => authService.loginWithToken(jwt),
    onSuccess: (userData, variables) => {
      queryClient.setQueryData(QUERY_KEYS.USER_SESSION, userData);
      invalidateQueries.auth();
      options?.onSuccess?.(userData, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// OAuth login mutation
export function useOAuthLogin(options?: UseMutationOptions<UserSession, any, { oauthToken: string; code: string; state: string }>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ oauthToken, code, state }) => authService.oauthLogin(oauthToken, code, state),
    onSuccess: (userData, variables) => {
      queryClient.setQueryData(QUERY_KEYS.USER_SESSION, userData);
      invalidateQueries.auth();
      options?.onSuccess?.(userData, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Register mutation
export function useRegister(options?: UseMutationOptions<APIResponse, any, RegisterDetails>) {
  return useMutation({
    mutationFn: (data: RegisterDetails) => authService.register(data),
    onSuccess: options?.onSuccess,
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Check session query
export function useSession(options?: UseQueryOptions<UserSession | null>) {
  return useQuery({
    queryKey: QUERY_KEYS.USER_SESSION,
    queryFn: () => authService.checkSession(),
    ...createQueryOptions.realtime,
    retry: false, // Don't retry session checks
    refetchInterval: 5 * 60 * 1000, // Check every 5 minutes
    ...options,
  });
}

// Logout mutation
export function useLogout(options?: UseMutationOptions<void, any, { isSysAdmin?: boolean }>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ isSysAdmin }) => authService.logout(isSysAdmin),
    onSuccess: (data, variables) => {
      // Clear all cached data on logout
      queryClient.clear();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Forgot password mutation
export function useForgotPassword(options?: UseMutationOptions<APIResponse, any, string>) {
  return useMutation({
    mutationFn: (email: string) => authService.forgotPassword(email),
    onSuccess: options?.onSuccess,
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Reset password mutation
export function useResetPassword(options?: UseMutationOptions<APIResponse, any, { token: string; password: string; email?: string }>) {
  return useMutation({
    mutationFn: ({ token, password, email }) => authService.resetPassword(token, password, email),
    onSuccess: options?.onSuccess,
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Export service instance for advanced usage
export { authService };

// Auth state helper hooks
export function useIsAuthenticated() {
  const { data: session } = useSession();
  return !!session;
}

export function useCurrentUser() {
  const { data: session, ...rest } = useSession();
  return { user: session, ...rest };
}

export function useIsAdmin() {
  const { data: session } = useSession();
  return !!session?.isSysAdmin;
}