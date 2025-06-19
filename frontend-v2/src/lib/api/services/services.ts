import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { getAPIClient } from '../client';
import { API_ENDPOINTS, QUERY_KEYS } from '../../constants/api';
import { createQueryOptions, invalidateQueries } from '../queryClient';
import {
  Service,
  ServiceType,
  APIListResponse,
  RequestOptions,
  UseMutationOptions,
  UseQueryOptions,
  ID,
} from '../../types/api';
import {
  ServiceSchema,
  ServiceTypeSchema,
  ServiceListResponseSchema,
  createListResponseSchema,
} from '../../validation/schemas';

// Services Service Class
export class ServicesService {
  private client = getAPIClient();
  
  // Get all services
  async getServices(options?: RequestOptions): Promise<APIListResponse<Service>> {
    const response = await this.client.getAll<APIListResponse<Service>>(
      API_ENDPOINTS.SYSTEM_SERVICE,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return ServiceListResponseSchema.parse(response);
  }
  
  // Get service by ID
  async getService(id: ID, options?: RequestOptions): Promise<Service> {
    const response = await this.client.getById<Service>(
      API_ENDPOINTS.SYSTEM_SERVICE,
      id,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return ServiceSchema.parse(response);
  }
  
  // Create new service
  async createService(data: Partial<Service>, options?: RequestOptions): Promise<Service> {
    const validatedData = ServiceSchema.omit({ 
      id: true, 
      created_date: true, 
      last_modified_date: true 
    }).parse(data);
    
    const response = await this.client.create<Service>(
      API_ENDPOINTS.SYSTEM_SERVICE,
      validatedData,
      {
        snackbarSuccess: 'Service created successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return ServiceSchema.parse(response);
  }
  
  // Update service
  async updateService(id: ID, data: Partial<Service>, options?: RequestOptions): Promise<Service> {
    const validatedData = ServiceSchema.partial().parse(data);
    
    const response = await this.client.update<Service>(
      API_ENDPOINTS.SYSTEM_SERVICE,
      id,
      validatedData,
      {
        snackbarSuccess: 'Service updated successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return ServiceSchema.parse(response);
  }
  
  // Delete service(s)
  async deleteService(id: ID | ID[], options?: RequestOptions): Promise<void> {
    await this.client.remove(
      API_ENDPOINTS.SYSTEM_SERVICE,
      id,
      {
        snackbarSuccess: Array.isArray(id) ? 'Services deleted successfully' : 'Service deleted successfully',
        snackbarError: 'server',
        ...options,
      }
    );
  }
  
  // Get all service types
  async getServiceTypes(options?: RequestOptions): Promise<APIListResponse<ServiceType>> {
    const response = await this.client.getAll<APIListResponse<ServiceType>>(
      API_ENDPOINTS.SERVICE_TYPE,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    const ServiceTypeListResponseSchema = createListResponseSchema(ServiceTypeSchema);
    return ServiceTypeListResponseSchema.parse(response);
  }
  
  // Test service connection
  async testServiceConnection(id: ID, options?: RequestOptions): Promise<{ success: boolean; message?: string }> {
    const response = await this.client.post<{ success: boolean; message?: string }>(
      `${API_ENDPOINTS.SYSTEM_SERVICE}/${id}`,
      null,
      {
        snackbarSuccess: 'Service connection test completed',
        snackbarError: 'server',
        additionalParams: { test: true },
        ...options,
      }
    );
    
    return response;
  }
  
  // Get service documentation
  async getServiceDocumentation(serviceName: string, options?: RequestOptions): Promise<any> {
    const response = await this.client.get(
      `${API_ENDPOINTS.API_DOCS}/${serviceName}`,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return response;
  }
  
  // Refresh service cache
  async refreshServiceCache(id: ID, options?: RequestOptions): Promise<void> {
    await this.client.post(
      `${API_ENDPOINTS.SYSTEM_SERVICE}/${id}`,
      null,
      {
        snackbarSuccess: 'Service cache refreshed successfully',
        snackbarError: 'server',
        additionalParams: { refresh: true },
        ...options,
      }
    );
  }
  
  // Get service reports
  async getServiceReports(options?: RequestOptions): Promise<any[]> {
    const response = await this.client.getAll(
      API_ENDPOINTS.SERVICE_REPORT,
      {
        snackbarError: 'server',
        ...options,
      }
    );
    
    return response;
  }
  
  // Bulk operations
  async bulkUpdateServices(services: Array<Partial<Service> & { id: ID }>, options?: RequestOptions): Promise<APIListResponse<Service>> {
    const validatedData = services.map(service => ServiceSchema.partial().parse(service));
    
    const response = await this.client.patch<APIListResponse<Service>>(
      API_ENDPOINTS.SYSTEM_SERVICE,
      { resource: validatedData },
      {
        snackbarSuccess: 'Services updated successfully',
        snackbarError: 'server',
        ...options,
      }
    );
    
    return ServiceListResponseSchema.parse(response);
  }
  
  // Enable/disable service
  async toggleServiceStatus(id: ID, isActive: boolean, options?: RequestOptions): Promise<Service> {
    return this.updateService(id, { is_active: isActive }, {
      snackbarSuccess: `Service ${isActive ? 'enabled' : 'disabled'} successfully`,
      ...options,
    });
  }
}

// Create service instance
const servicesService = new ServicesService();

// React Query Hooks

// Get all services query
export function useServices(options?: RequestOptions & UseQueryOptions<APIListResponse<Service>>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: [...QUERY_KEYS.SERVICES, requestOptions],
    queryFn: () => servicesService.getServices(requestOptions),
    ...createQueryOptions.config,
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

// Get service by ID query
export function useService(id: ID, options?: RequestOptions & UseQueryOptions<Service>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: QUERY_KEYS.SERVICE(id),
    queryFn: () => servicesService.getService(id, requestOptions),
    ...createQueryOptions.config,
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

// Get service types query
export function useServiceTypes(options?: RequestOptions & UseQueryOptions<APIListResponse<ServiceType>>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: [...QUERY_KEYS.SERVICE_TYPES, requestOptions],
    queryFn: () => servicesService.getServiceTypes(requestOptions),
    ...createQueryOptions.static,
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

// Get service documentation query
export function useServiceDocumentation(serviceName: string, options?: RequestOptions & UseQueryOptions<any>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: [...QUERY_KEYS.API_DOCS, serviceName],
    queryFn: () => servicesService.getServiceDocumentation(serviceName, requestOptions),
    ...createQueryOptions.static,
    enabled: enabled && !!serviceName,
    retry,
    staleTime,
    gcTime,
    refetchOnWindowFocus,
    refetchOnMount,
    refetchOnReconnect,
    select,
  });
}

// Get service reports query
export function useServiceReports(options?: RequestOptions & UseQueryOptions<any[]>) {
  const { enabled, retry, staleTime, gcTime, refetchOnWindowFocus, refetchOnMount, refetchOnReconnect, select, onSuccess, onError, onSettled, ...requestOptions } = options || {};
  
  return useQuery({
    queryKey: [...QUERY_KEYS.SERVICE_REPORTS, requestOptions],
    queryFn: () => servicesService.getServiceReports(requestOptions),
    ...createQueryOptions.config,
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

// Create service mutation
export function useCreateService(options?: UseMutationOptions<Service, any, Partial<Service>>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: Partial<Service>) => servicesService.createService(data),
    onSuccess: (data, variables) => {
      invalidateQueries.services();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Update service mutation
export function useUpdateService(options?: UseMutationOptions<Service, any, { id: ID; data: Partial<Service> }>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }) => servicesService.updateService(id, data),
    onSuccess: (data, variables) => {
      queryClient.setQueryData(QUERY_KEYS.SERVICE(variables.id), data);
      invalidateQueries.services();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Delete service mutation
export function useDeleteService(options?: UseMutationOptions<void, any, ID | ID[]>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: ID | ID[]) => servicesService.deleteService(id),
    onSuccess: (data, variables) => {
      if (Array.isArray(variables)) {
        variables.forEach(id => {
          queryClient.removeQueries({ queryKey: QUERY_KEYS.SERVICE(id) });
        });
      } else {
        queryClient.removeQueries({ queryKey: QUERY_KEYS.SERVICE(variables) });
      }
      invalidateQueries.services();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Test service connection mutation
export function useTestServiceConnection(options?: UseMutationOptions<{ success: boolean; message?: string }, any, ID>) {
  return useMutation({
    mutationFn: (id: ID) => servicesService.testServiceConnection(id),
    onSuccess: options?.onSuccess,
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Refresh service cache mutation
export function useRefreshServiceCache(options?: UseMutationOptions<void, any, ID>) {
  return useMutation({
    mutationFn: (id: ID) => servicesService.refreshServiceCache(id),
    onSuccess: (data, variables) => {
      // Invalidate the specific service to refetch fresh data
      invalidateQueries.service(variables);
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Bulk update services mutation
export function useBulkUpdateServices(options?: UseMutationOptions<APIListResponse<Service>, any, Array<Partial<Service> & { id: ID }>>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (services: Array<Partial<Service> & { id: ID }>) => servicesService.bulkUpdateServices(services),
    onSuccess: (data, variables) => {
      variables.forEach(service => {
        const updatedService = data.resource?.find(s => s.id === service.id);
        if (updatedService) {
          queryClient.setQueryData(QUERY_KEYS.SERVICE(service.id), updatedService);
        }
      });
      invalidateQueries.services();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Toggle service status mutation
export function useToggleServiceStatus(options?: UseMutationOptions<Service, any, { id: ID; isActive: boolean }>) {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, isActive }) => servicesService.toggleServiceStatus(id, isActive),
    onSuccess: (data, variables) => {
      queryClient.setQueryData(QUERY_KEYS.SERVICE(variables.id), data);
      invalidateQueries.services();
      options?.onSuccess?.(data, variables);
    },
    onError: options?.onError,
    onSettled: options?.onSettled,
  });
}

// Export service instance
export { servicesService };