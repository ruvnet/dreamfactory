// Base API Response Types
export interface APIResponse<T = unknown> {
  success?: boolean;
  resource?: T;
  meta?: {
    count: number;
    schema?: any[];
  };
}

export interface APIListResponse<T = unknown> extends APIResponse<T[]> {
  resource: T[];
  meta: {
    count: number;
    schema?: any[];
  };
}

export interface APIError {
  error: {
    code: string | number;
    message: string;
    status_code: number;
    context?: string | {
      error: any[];
      resource: APIError[];
    };
  };
}

// Request Options Interface
export interface RequestOptions {
  // Query Parameters
  limit?: number;
  offset?: number;
  filter?: string;
  sort?: string;
  fields?: string;
  related?: string;
  include_count?: boolean;
  refresh?: boolean;
  
  // Additional Parameters
  additionalParams?: Record<string, any>;
  
  // Headers
  additionalHeaders?: Record<string, string>;
  contentType?: string;
  
  // UI Control
  showSpinner?: boolean;
  snackbarSuccess?: string;
  snackbarError?: string;
  includeCacheControl?: boolean;
}

// Authentication Types
export interface LoginCredentials {
  email: string;
  password: string;
  remember_me?: boolean;
}

export interface RegisterDetails {
  name: string;
  first_name: string;
  last_name: string;
  email: string;
  password: string;
  phone?: string;
  security_question?: string;
  security_answer?: string;
  adldap?: string;
}

export interface UserSession {
  id: number;
  name: string;
  username?: string;
  first_name: string;
  last_name: string;
  email: string;
  is_active: boolean;
  phone?: string;
  security_question?: string;
  confirmed_email?: boolean;
  oauth_provider?: string;
  created_date: string;
  last_modified_date: string;
  session_token: string;
  session_id: string;
  role: UserRole;
  app_groups?: AppGroup[];
  no_group_access?: boolean;
  adldap?: string;
  saml?: string;
  isSysAdmin?: boolean;
  host?: string;
}

export interface UserRole {
  id: number;
  name: string;
  description?: string;
  is_active: boolean;
  role_service_access_by_role_id?: RoleServiceAccess[];
  role_adldap_by_role_id?: any[];
  role_saml_by_role_id?: any[];
  created_date: string;
  last_modified_date: string;
}

export interface RoleServiceAccess {
  id: number;
  role_id: number;
  service_id: number;
  component: string;
  verb_mask: number;
  requestor_type: number;
  filters?: any[];
  filter_op: string;
  created_date: string;
  last_modified_date: string;
  service_by_service_id: Service;
}

export interface AppGroup {
  id: number;
  name: string;
  description?: string;
  is_active: boolean;
  created_date: string;
  last_modified_date: string;
}

// Service Types
export interface Service {
  id: number;
  name: string;
  label: string;
  description?: string;
  is_active: boolean;
  type: string;
  mutable?: boolean;
  deletable?: boolean;
  created_date: string;
  last_modified_date: string;
  service_doc_by_service_id?: ServiceDoc[];
  config?: ServiceConfig;
}

export interface ServiceDoc {
  service_id: number;
  format: string;
  content: string;
}

export interface ServiceConfig {
  service_id: number;
  [key: string]: any;
}

export interface ServiceType {
  name: string;
  label: string;
  description?: string;
  group: string;
  singleton?: boolean;
  dependencies?: string[];
  subscription_required?: boolean;
}

// Application Types
export interface App {
  id: number;
  name: string;
  description?: string;
  is_active: boolean;
  url?: string;
  is_url_external?: boolean;
  import_url?: string;
  storage_service_id?: number;
  storage_container?: string;
  path?: string;
  api_key: string;
  role_id?: number;
  created_date: string;
  last_modified_date: string;
  app_group_by_app_id?: AppGroup[];
  role_by_role_id?: UserRole;
}

// Role Types
export interface Role {
  id: number;
  name: string;
  description?: string;
  is_active: boolean;
  created_date: string;
  last_modified_date: string;
  role_service_access_by_role_id?: RoleServiceAccess[];
  role_adldap_by_role_id?: any[];
  role_saml_by_role_id?: any[];
  user_by_role_id?: UserSession[];
  app_by_role_id?: App[];
}

// User Types
export interface User {
  id: number;
  name: string;
  username?: string;
  first_name: string;
  last_name: string;
  email: string;
  is_active: boolean;
  phone?: string;
  security_question?: string;
  security_answer?: string;
  confirmed_email?: boolean;
  oauth_provider?: string;
  created_date: string;
  last_modified_date: string;
  role_id?: number;
  role_by_role_id?: UserRole;
  user_app_role_by_user_id?: UserAppRole[];
  user_lookup_by_user_id?: UserLookup[];
  adldap?: string;
  saml?: string;
}

export interface UserAppRole {
  id: number;
  user_id: number;
  app_id: number;
  role_id: number;
  created_date: string;
  last_modified_date: string;
  app_by_app_id: App;
  role_by_role_id: UserRole;
}

export interface UserLookup {
  id: number;
  user_id: number;
  name: string;
  value?: string;
  private: boolean;
  description?: string;
  created_date: string;
  last_modified_date: string;
}

// System Types
export interface SystemInfo {
  platform: {
    name: string;
    version: string;
    build_date?: string;
    commit_hash?: string;
  };
  server_os: {
    name: string;
    version: string;
    architecture: string;
  };
  php: {
    version: string;
    extensions: string[];
  };
  database: {
    driver: string;
    version: string;
  };
  cache: {
    driver: string;
  };
  [key: string]: any;
}

// Configuration Types
export interface EmailTemplate {
  id: number;
  name: string;
  description?: string;
  to: string[];
  cc?: string[];
  bcc?: string[];
  subject: string;
  body_text?: string;
  body_html?: string;
  from_name?: string;
  from_email?: string;
  reply_to_name?: string;
  reply_to_email?: string;
  defaults?: any[];
  created_date: string;
  last_modified_date: string;
}

export interface LookupKey {
  id: number;
  name: string;
  value?: string;
  private: boolean;
  description?: string;
  created_date: string;
  last_modified_date: string;
}

export interface CorsConfig {
  id: number;
  path: string;
  origin: string;
  header: string;
  method: string;
  max_age?: number;
  supports_credentials?: boolean;
  created_date: string;
  last_modified_date: string;
}

// Event & Script Types
export interface Event {
  name: string;
  label: string;
  description?: string;
  service_name?: string;
  resource?: string;
  verb?: string;
}

export interface EventScript {
  id: number;
  name: string;
  type: string;
  is_active: boolean;
  source: string;
  config?: any;
  created_date: string;
  last_modified_date: string;
}

export interface ScriptType {
  name: string;
  label: string;
  description?: string;
  sandboxed?: boolean;
  extensions?: string[];
}

// Scheduler Types
export interface SchedulerTask {
  id: number;
  name: string;
  description?: string;
  is_active: boolean;
  verb: string;
  service: string;
  resource?: string;
  payload?: any;
  frequency: number;
  frequency_type: string;
  start_date: string;
  end_date?: string;
  last_run?: string;
  next_run?: string;
  created_date: string;
  last_modified_date: string;
}

// Limit Types
export interface Limit {
  id: number;
  name: string;
  description?: string;
  is_active: boolean;
  type: string;
  key_text: string;
  rate: number;
  period: string;
  user_id?: number;
  role_id?: number;
  service_id?: number;
  verb_mask?: number;
  created_date: string;
  last_modified_date: string;
}

// Files Types
export interface FileInfo {
  name: string;
  path: string;
  type: 'file' | 'folder';
  size?: number;
  last_modified?: string;
  content_type?: string;
}

// API Docs Types
export interface ApiDoc {
  service_name: string;
  paths: Record<string, any>;
  definitions?: Record<string, any>;
  info?: {
    title: string;
    description?: string;
    version: string;
  };
}

// Utility Types
export type ID = string | number;
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

// Query Hook Options
export interface UseQueryOptions<TData = unknown, TError = APIError> {
  enabled?: boolean;
  retry?: number | boolean;
  staleTime?: number;
  cacheTime?: number;
  refetchOnWindowFocus?: boolean;
  refetchOnMount?: boolean;
  refetchOnReconnect?: boolean;
  select?: (data: TData) => any;
  onSuccess?: (data: TData) => void;
  onError?: (error: TError) => void;
  onSettled?: (data: TData | undefined, error: TError | null) => void;
}

export interface UseMutationOptions<TData = unknown, TError = APIError, TVariables = void> {
  onSuccess?: (data: TData, variables: TVariables) => void;
  onError?: (error: TError, variables: TVariables) => void;
  onSettled?: (data: TData | undefined, error: TError | null, variables: TVariables) => void;
  onMutate?: (variables: TVariables) => void;
}