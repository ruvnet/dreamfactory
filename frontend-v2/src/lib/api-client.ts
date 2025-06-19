import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios'
import { useAuthStore } from '@/stores'

// API base configuration
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000/api/v2'
const API_TIMEOUT = 30000 // 30 seconds

// Create axios instance
export const apiClient: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: API_TIMEOUT,
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  },
})

// Request interceptor
apiClient.interceptors.request.use(
  (config) => {
    // Get token from auth store
    const token = useAuthStore.getState().token
    
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }

    // Add request timestamp for debugging
    config.metadata = { requestStartTime: Date.now() }
    
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    // Log response time in development
    if (import.meta.env.DEV) {
      const requestTime = Date.now() - response.config.metadata?.requestStartTime
      console.log(`API Request to ${response.config.url} took ${requestTime}ms`)
    }
    
    return response
  },
  (error) => {
    // Handle common errors
    if (error.response) {
      const { status, data } = error.response
      
      switch (status) {
        case 401:
          // Unauthorized - clear auth state and redirect to login
          useAuthStore.getState().logout()
          window.location.href = '/login'
          break
          
        case 403:
          // Forbidden - show error message
          console.error('Access forbidden:', data.message)
          break
          
        case 404:
          // Not found
          console.error('Resource not found:', error.config?.url)
          break
          
        case 422:
          // Validation errors
          console.error('Validation errors:', data.errors)
          break
          
        case 500:
          // Server error
          console.error('Server error:', data.message)
          break
          
        default:
          console.error('API Error:', error.message)
      }
    } else if (error.request) {
      // Network error
      console.error('Network error:', error.message)
    } else {
      // Request setup error
      console.error('Request error:', error.message)
    }
    
    return Promise.reject(error)
  }
)

// API helper functions
export const api = {
  get: <T>(url: string, config?: AxiosRequestConfig) => 
    apiClient.get<T>(url, config).then(response => response.data),
    
  post: <T>(url: string, data?: any, config?: AxiosRequestConfig) => 
    apiClient.post<T>(url, data, config).then(response => response.data),
    
  put: <T>(url: string, data?: any, config?: AxiosRequestConfig) => 
    apiClient.put<T>(url, data, config).then(response => response.data),
    
  patch: <T>(url: string, data?: any, config?: AxiosRequestConfig) => 
    apiClient.patch<T>(url, data, config).then(response => response.data),
    
  delete: <T>(url: string, config?: AxiosRequestConfig) => 
    apiClient.delete<T>(url, config).then(response => response.data),
}

// Type declarations for request metadata
declare module 'axios' {
  export interface AxiosRequestConfig {
    metadata?: {
      requestStartTime: number
    }
  }
}