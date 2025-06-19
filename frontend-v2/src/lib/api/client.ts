import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import { API_CONFIG, HTTP_HEADERS } from '../constants/api';
import { APIError, RequestOptions } from '../types/api';

// Environment configuration interface
interface APIClientConfig {
  baseURL: string;
  apiKey: string;
  timeout?: number;
}

// Create the API client class
export class APIClient {
  private instance: AxiosInstance;
  private sessionToken: string | null = null;
  
  constructor(config: APIClientConfig) {
    this.instance = axios.create({
      baseURL: config.baseURL,
      timeout: config.timeout || API_CONFIG.DEFAULT_TIMEOUT,
      headers: {
        [HTTP_HEADERS.API_KEY]: config.apiKey,
        [HTTP_HEADERS.CONTENT_TYPE]: 'application/json',
        [HTTP_HEADERS.CACHE_CONTROL]: 'no-cache, private',
      },
    });
    
    this.setupInterceptors();
  }
  
  private setupInterceptors() {
    // Request interceptor
    this.instance.interceptors.request.use(
      (config) => {
        // Add session token if available
        if (this.sessionToken && config.headers) {
          config.headers[HTTP_HEADERS.SESSION_TOKEN] = this.sessionToken;
        }
        
        // Add custom headers for loading, snackbar, etc.
        if (config.metadata?.showSpinner !== false) {
          config.headers = config.headers || {};
          config.headers[HTTP_HEADERS.SHOW_LOADING] = '';
        }
        
        if (config.metadata?.snackbarSuccess) {
          config.headers = config.headers || {};
          config.headers[HTTP_HEADERS.SNACKBAR_SUCCESS] = config.metadata.snackbarSuccess;
        }
        
        if (config.metadata?.snackbarError) {
          config.headers = config.headers || {};
          config.headers[HTTP_HEADERS.SNACKBAR_ERROR] = config.metadata.snackbarError;
        }
        
        return config;
      },
      (error) => {
        return Promise.reject(this.handleError(error));
      }
    );
    
    // Response interceptor
    this.instance.interceptors.response.use(
      (response) => {
        return response;
      },
      (error) => {
        return Promise.reject(this.handleError(error));
      }
    );
  }
  
  private handleError(error: any): APIError {
    if (error.response?.data?.error) {
      return error.response.data as APIError;
    }
    
    // Fallback error structure
    return {
      error: {
        code: error.response?.status || 'UNKNOWN_ERROR',
        message: error.message || 'An unknown error occurred',
        status_code: error.response?.status || 500,
        context: error.response?.data || undefined,
      },
    };
  }
  
  // Set session token
  setSessionToken(token: string | null) {
    this.sessionToken = token;
  }
  
  // Get session token
  getSessionToken(): string | null {
    return this.sessionToken;
  }
  
  // Build request options
  private buildRequestConfig(options?: Partial<RequestOptions>): AxiosRequestConfig & { metadata?: any } {
    const config: AxiosRequestConfig & { metadata?: any } = {
      params: {},
      headers: {},
      metadata: {},
    };
    
    if (!options) return config;
    
    // Query parameters
    if (options.limit !== undefined) config.params.limit = options.limit;
    if (options.offset !== undefined) config.params.offset = options.offset;
    if (options.filter) config.params.filter = options.filter;
    if (options.sort) config.params.sort = options.sort;
    if (options.fields) config.params.fields = options.fields;
    if (options.related) config.params.related = options.related;
    if (options.include_count !== undefined) config.params.include_count = options.include_count;
    if (options.refresh) config.params.refresh = options.refresh;
    
    // Additional parameters
    if (options.additionalParams) {
      Object.assign(config.params, options.additionalParams);
    }
    
    // Headers
    if (options.contentType) {
      config.headers![HTTP_HEADERS.CONTENT_TYPE] = options.contentType;
    }
    
    if (options.additionalHeaders) {
      Object.assign(config.headers, options.additionalHeaders);
    }
    
    if (options.includeCacheControl !== false) {
      config.headers![HTTP_HEADERS.CACHE_CONTROL] = 'no-cache, private';
    }
    
    // Metadata for interceptors
    config.metadata = {
      showSpinner: options.showSpinner,
      snackbarSuccess: options.snackbarSuccess,
      snackbarError: options.snackbarError,
    };
    
    return config;
  }
  
  // HTTP Methods
  async get<T>(url: string, options?: Partial<RequestOptions>): Promise<T> {
    const config = this.buildRequestConfig(options);
    const response: AxiosResponse<T> = await this.instance.get(url, config);
    return response.data;
  }
  
  async post<T>(url: string, data?: any, options?: Partial<RequestOptions>): Promise<T> {
    const config = this.buildRequestConfig(options);
    const response: AxiosResponse<T> = await this.instance.post(url, data, config);
    return response.data;
  }
  
  async put<T>(url: string, data?: any, options?: Partial<RequestOptions>): Promise<T> {
    const config = this.buildRequestConfig(options);
    const response: AxiosResponse<T> = await this.instance.put(url, data, config);
    return response.data;
  }
  
  async patch<T>(url: string, data?: any, options?: Partial<RequestOptions>): Promise<T> {
    const config = this.buildRequestConfig(options);
    const response: AxiosResponse<T> = await this.instance.patch(url, data, config);
    return response.data;
  }
  
  async delete<T>(url: string, options?: Partial<RequestOptions>): Promise<T> {
    const config = this.buildRequestConfig(options);
    const response: AxiosResponse<T> = await this.instance.delete(url, config);
    return response.data;
  }
  
  // Specialized methods for DreamFactory patterns
  async getAll<T>(url: string, options?: Partial<RequestOptions>) {
    const defaultOptions: Partial<RequestOptions> = {
      limit: 50,
      offset: 0,
      include_count: true,
      ...options,
    };
    return this.get<T>(url, defaultOptions);
  }
  
  async getById<T>(url: string, id: string | number, options?: Partial<RequestOptions>) {
    const defaultOptions: Partial<RequestOptions> = {
      snackbarError: 'server',
      ...options,
    };
    return this.get<T>(`${url}/${id}`, defaultOptions);
  }
  
  async create<T>(url: string, data: any, options?: Partial<RequestOptions>) {
    return this.post<T>(url, data, options);
  }
  
  async update<T>(url: string, id: string | number, data: any, options?: Partial<RequestOptions>) {
    return this.put<T>(`${url}/${id}`, data, options);
  }
  
  async remove<T>(url: string, id: string | number | Array<string | number>, options?: Partial<RequestOptions>) {
    const defaultOptions: Partial<RequestOptions> = {
      snackbarError: 'server',
      ...options,
    };
    
    const deleteUrl = Array.isArray(id)
      ? `${url}?ids=${id.join(',')}`
      : `${url}/${id}`;
      
    return this.delete<T>(deleteUrl, defaultOptions);
  }
  
  // File operations
  async uploadFile(url: string, files: FileList, options?: Partial<RequestOptions>) {
    const formData = new FormData();
    Array.from(files).forEach((file, index) => {
      formData.append('files', file);
    });
    
    const uploadOptions: Partial<RequestOptions> = {
      contentType: 'multipart/form-data',
      snackbarError: 'server',
      ...options,
    };
    
    // Remove Content-Type header to let browser set it with boundary
    const config = this.buildRequestConfig(uploadOptions);
    delete config.headers![HTTP_HEADERS.CONTENT_TYPE];
    
    const response: AxiosResponse = await this.instance.post(url, formData, config);
    return response.data;
  }
  
  async downloadFile(url: string, options?: Partial<RequestOptions>) {
    const config = this.buildRequestConfig({
      snackbarError: 'server',
      ...options,
    });
    
    const response: AxiosResponse<Blob> = await this.instance.get(url, {
      ...config,
      responseType: 'blob',
    });
    
    return response.data;
  }
  
  async downloadJson(url: string, options?: Partial<RequestOptions>) {
    const data = await this.get(url, {
      snackbarError: 'server',
      ...options,
    });
    
    return JSON.stringify(data, null, 2);
  }
  
  // Legacy delete method (for services that require POST with X-HTTP-Method-Override)
  async legacyDelete<T>(url: string, endpoint: string, options?: Partial<RequestOptions>) {
    const config = this.buildRequestConfig({
      snackbarError: 'server',
      ...options,
    });
    
    config.headers![HTTP_HEADERS.HTTP_METHOD_OVERRIDE] = 'DELETE';
    
    const response: AxiosResponse<T> = await this.instance.post(`${url}/${endpoint}`, null, config);
    return response.data;
  }
}

// Default API client instance
let defaultClient: APIClient | null = null;

// Initialize the default client
export function initializeAPIClient(config: APIClientConfig) {
  defaultClient = new APIClient(config);
  return defaultClient;
}

// Get the default client instance
export function getAPIClient(): APIClient {
  if (!defaultClient) {
    throw new Error('API client not initialized. Call initializeAPIClient() first.');
  }
  return defaultClient;
}

// APIClient is already exported above

// Extend axios config type to include our metadata
declare module 'axios' {
  interface AxiosRequestConfig {
    metadata?: {
      showSpinner?: boolean;
      snackbarSuccess?: string;
      snackbarError?: string;
    };
  }
}