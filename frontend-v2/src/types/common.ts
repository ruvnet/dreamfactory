/**
 * Common type definitions used throughout the application
 */

export interface BaseEntity {
  id: string
  created_at?: string
  updated_at?: string
}

export interface ApiResponse<T> {
  data: T
  status: number
  message?: string
  errors?: Record<string, string[]>
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  meta: {
    current_page: number
    last_page: number
    per_page: number
    total: number
    from: number
    to: number
  }
}

export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

export interface TableColumn<T = any> {
  key: keyof T
  label: string
  sortable?: boolean
  width?: string
  render?: (value: any, record: T) => React.ReactNode
}

export type LoadingState = 'idle' | 'loading' | 'success' | 'error'

export interface AsyncState<T> {
  data: T | null
  loading: boolean
  error: string | null
}

export type Theme = 'light' | 'dark' | 'system'

export type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

export type Variant = 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'outline'