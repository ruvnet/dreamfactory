// API Configuration Constants
export const API_CONFIG = {
  BASE_URL: '/api/v2',
  DEFAULT_TIMEOUT: 30000,
  RETRY_COUNT: 3,
  RETRY_DELAY: 1000,
} as const;

// API Endpoints based on DreamFactory API structure
export const API_ENDPOINTS = {
  // Authentication & Sessions
  USER_SESSION: `${API_CONFIG.BASE_URL}/user/session`,
  ADMIN_SESSION: `${API_CONFIG.BASE_URL}/system/admin/session`,
  USER_PASSWORD: `${API_CONFIG.BASE_URL}/user/password`,
  ADMIN_PASSWORD: `${API_CONFIG.BASE_URL}/system/admin/password`,
  REGISTER: `${API_CONFIG.BASE_URL}/user/register`,
  
  // System
  SYSTEM: `${API_CONFIG.BASE_URL}/system`,
  ENVIRONMENT: `${API_CONFIG.BASE_URL}/system/environment`,
  
  // User Management
  ADMIN_PROFILE: `${API_CONFIG.BASE_URL}/system/admin/profile`,
  USER_PROFILE: `${API_CONFIG.BASE_URL}/user/profile`,
  SYSTEM_ADMIN: `${API_CONFIG.BASE_URL}/system/admin`,
  SYSTEM_USER: `${API_CONFIG.BASE_URL}/system/user`,
  
  // Applications & Services
  APP: `${API_CONFIG.BASE_URL}/system/app`,
  SYSTEM_SERVICE: `${API_CONFIG.BASE_URL}/system/service`,
  SERVICE_TYPE: `${API_CONFIG.BASE_URL}/system/service_type`,
  SERVICE_REPORT: `${API_CONFIG.BASE_URL}/system/service_report`,
  
  // Roles & Security
  ROLES: `${API_CONFIG.BASE_URL}/system/role`,
  LIMITS: `${API_CONFIG.BASE_URL}/system/limit`,
  LIMIT_CACHE: `${API_CONFIG.BASE_URL}/system/limit_cache`,
  SYSTEM_CORS: `${API_CONFIG.BASE_URL}/system/cors`,
  
  // API Documentation
  API_DOCS: `${API_CONFIG.BASE_URL}/api_docs`,
  
  // Event Management
  SYSTEM_EVENT: `${API_CONFIG.BASE_URL}/system/event`,
  EVENT_SCRIPT: `${API_CONFIG.BASE_URL}/system/event_script`,
  SCRIPT_TYPE: `${API_CONFIG.BASE_URL}/system/script_type`,
  SCHEDULER: `${API_CONFIG.BASE_URL}/system/scheduler`,
  
  // Configuration
  SYSTEM_CACHE: `${API_CONFIG.BASE_URL}/system/cache`,
  EMAIL_TEMPLATES: `${API_CONFIG.BASE_URL}/system/email_template`,
  LOOKUP_KEYS: `${API_CONFIG.BASE_URL}/system/lookup`,
  
  // Files & Storage
  FILES: `${API_CONFIG.BASE_URL}/files`,
  LOGS: `${API_CONFIG.BASE_URL}/logs`,
  
  // External APIs
  GITHUB_REPO: 'https://api.github.com/repos',
  SUBSCRIPTION_DATA: 'https://updates.dreamfactory.com/check',
} as const;

// HTTP Headers
export const HTTP_HEADERS = {
  API_KEY: 'X-DreamFactory-Api-Key',
  SESSION_TOKEN: 'X-DreamFactory-Session-Token',
  CONTENT_TYPE: 'Content-Type',
  AUTHORIZATION: 'Authorization',
  CACHE_CONTROL: 'Cache-Control',
  SHOW_LOADING: 'show-loading',
  SNACKBAR_SUCCESS: 'snackbar-success',
  SNACKBAR_ERROR: 'snackbar-error',
  HTTP_METHOD_OVERRIDE: 'X-Http-Method',
} as const;

// Query Keys for React Query
export const QUERY_KEYS = {
  // Authentication
  USER_SESSION: ['user', 'session'] as const,
  ADMIN_SESSION: ['admin', 'session'] as const,
  
  // Users & Admins
  USERS: ['users'] as const,
  USER: (id: string | number) => ['users', id] as const,
  ADMINS: ['admins'] as const,
  ADMIN: (id: string | number) => ['admins', id] as const,
  USER_PROFILE: ['user', 'profile'] as const,
  ADMIN_PROFILE: ['admin', 'profile'] as const,
  
  // Applications
  APPS: ['apps'] as const,
  APP: (id: string | number) => ['apps', id] as const,
  
  // Services
  SERVICES: ['services'] as const,
  SERVICE: (id: string | number) => ['services', id] as const,
  SERVICE_TYPES: ['service-types'] as const,
  SERVICE_REPORTS: ['service-reports'] as const,
  
  // Roles & Security
  ROLES: ['roles'] as const,
  ROLE: (id: string | number) => ['roles', id] as const,
  LIMITS: ['limits'] as const,
  LIMIT: (id: string | number) => ['limits', id] as const,
  CORS_CONFIG: ['cors-config'] as const,
  
  // Schema
  DATABASES: ['databases'] as const,
  DATABASE: (name: string) => ['databases', name] as const,
  TABLES: (dbName: string) => ['databases', dbName, 'tables'] as const,
  TABLE: (dbName: string, tableName: string) => ['databases', dbName, 'tables', tableName] as const,
  
  // Configuration
  SYSTEM_INFO: ['system', 'info'] as const,
  SYSTEM_CACHE: ['system', 'cache'] as const,
  EMAIL_TEMPLATES: ['email-templates'] as const,
  EMAIL_TEMPLATE: (id: string | number) => ['email-templates', id] as const,
  LOOKUP_KEYS: ['lookup-keys'] as const,
  
  // Event Scripts
  EVENTS: ['events'] as const,
  EVENT_SCRIPTS: ['event-scripts'] as const,
  EVENT_SCRIPT: (id: string | number) => ['event-scripts', id] as const,
  SCRIPT_TYPES: ['script-types'] as const,
  SCHEDULER: ['scheduler'] as const,
  SCHEDULER_TASK: (id: string | number) => ['scheduler', id] as const,
  
  // Files
  FILES: (path?: string) => path ? ['files', path] as const : ['files'] as const,
  LOGS: ['logs'] as const,
  
  // API Docs
  API_DOCS: ['api-docs'] as const,
} as const;

// Cache Times (in milliseconds)
export const CACHE_TIME = {
  SHORT: 5 * 60 * 1000, // 5 minutes
  MEDIUM: 15 * 60 * 1000, // 15 minutes
  LONG: 60 * 60 * 1000, // 1 hour
  VERY_LONG: 24 * 60 * 60 * 1000, // 24 hours
} as const;

// Stale Times (in milliseconds)
export const STALE_TIME = {
  IMMEDIATE: 0,
  SHORT: 30 * 1000, // 30 seconds
  MEDIUM: 5 * 60 * 1000, // 5 minutes
  LONG: 15 * 60 * 1000, // 15 minutes
} as const;