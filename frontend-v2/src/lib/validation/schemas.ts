import { z } from 'zod';

// Base Schemas
export const BaseResponseSchema = z.object({
  success: z.boolean().optional(),
  resource: z.unknown().optional(),
  meta: z.object({
    count: z.number(),
    schema: z.array(z.unknown()).optional(),
  }).optional(),
});

export const APIErrorSchema = z.object({
  error: z.object({
    code: z.union([z.string(), z.number()]),
    message: z.string(),
    status_code: z.number(),
    context: z.union([
      z.string(),
      z.object({
        error: z.array(z.unknown()),
        resource: z.array(z.unknown()),
      }),
    ]).optional(),
  }),
});

// Authentication Schemas
export const LoginCredentialsSchema = z.object({
  email: z.string().email('Please enter a valid email address'),
  password: z.string().min(1, 'Password is required'),
  remember_me: z.boolean().optional(),
});

export const RegisterDetailsSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  first_name: z.string().min(1, 'First name is required'),
  last_name: z.string().min(1, 'Last name is required'),
  email: z.string().email('Please enter a valid email address'),
  password: z.string()
    .min(8, 'Password must be at least 8 characters')
    .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/, 'Password must contain at least one lowercase letter, one uppercase letter, and one number'),
  phone: z.string().optional(),
  security_question: z.string().optional(),
  security_answer: z.string().optional(),
  adldap: z.string().optional(),
});

export const UserRoleSchema = z.object({
  id: z.number(),
  name: z.string(),
  description: z.string().optional(),
  is_active: z.boolean(),
  role_service_access_by_role_id: z.array(z.unknown()).optional(),
  role_adldap_by_role_id: z.array(z.unknown()).optional(),
  role_saml_by_role_id: z.array(z.unknown()).optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

export const AppGroupSchema = z.object({
  id: z.number(),
  name: z.string(),
  description: z.string().optional(),
  is_active: z.boolean(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

export const UserSessionSchema = z.object({
  id: z.number(),
  name: z.string(),
  username: z.string().optional(),
  first_name: z.string(),
  last_name: z.string(),
  email: z.string().email(),
  is_active: z.boolean(),
  phone: z.string().optional(),
  security_question: z.string().optional(),
  confirmed_email: z.boolean().optional(),
  oauth_provider: z.string().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
  session_token: z.string(),
  session_id: z.string(),
  role: UserRoleSchema,
  app_groups: z.array(AppGroupSchema).optional(),
  no_group_access: z.boolean().optional(),
  adldap: z.string().optional(),
  saml: z.string().optional(),
  isSysAdmin: z.boolean().optional(),
  host: z.string().optional(),
});

// Service Schemas
export const ServiceConfigSchema = z.object({
  service_id: z.number(),
}).catchall(z.unknown());

export const ServiceDocSchema = z.object({
  service_id: z.number(),
  format: z.string(),
  content: z.string(),
});

export const ServiceSchema = z.object({
  id: z.number(),
  name: z.string(),
  label: z.string(),
  description: z.string().optional(),
  is_active: z.boolean(),
  type: z.string(),
  mutable: z.boolean().optional(),
  deletable: z.boolean().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
  service_doc_by_service_id: z.array(ServiceDocSchema).optional(),
  config: ServiceConfigSchema.optional(),
});

export const ServiceTypeSchema = z.object({
  name: z.string(),
  label: z.string(),
  description: z.string().optional(),
  group: z.string(),
  singleton: z.boolean().optional(),
  dependencies: z.array(z.string()).optional(),
  subscription_required: z.boolean().optional(),
});

// Application Schemas
export const AppSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'App name is required'),
  description: z.string().optional(),
  is_active: z.boolean(),
  url: z.string().optional(),
  is_url_external: z.boolean().optional(),
  import_url: z.string().optional(),
  storage_service_id: z.number().optional(),
  storage_container: z.string().optional(),
  path: z.string().optional(),
  api_key: z.string(),
  role_id: z.number().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
  app_group_by_app_id: z.array(AppGroupSchema).optional(),
  role_by_role_id: UserRoleSchema.optional(),
});

// Role Schemas
export const RoleServiceAccessSchema = z.object({
  id: z.number(),
  role_id: z.number(),
  service_id: z.number(),
  component: z.string(),
  verb_mask: z.number(),
  requestor_type: z.number(),
  filters: z.array(z.unknown()).optional(),
  filter_op: z.string(),
  created_date: z.string(),
  last_modified_date: z.string(),
  service_by_service_id: ServiceSchema,
});

export const RoleSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Role name is required'),
  description: z.string().optional(),
  is_active: z.boolean(),
  created_date: z.string(),
  last_modified_date: z.string(),
  role_service_access_by_role_id: z.array(RoleServiceAccessSchema).optional(),
  role_adldap_by_role_id: z.array(z.unknown()).optional(),
  role_saml_by_role_id: z.array(z.unknown()).optional(),
  user_by_role_id: z.array(UserSessionSchema).optional(),
  app_by_role_id: z.array(AppSchema).optional(),
});

// User Schemas
export const UserAppRoleSchema = z.object({
  id: z.number(),
  user_id: z.number(),
  app_id: z.number(),
  role_id: z.number(),
  created_date: z.string(),
  last_modified_date: z.string(),
  app_by_app_id: AppSchema,
  role_by_role_id: UserRoleSchema,
});

export const UserLookupSchema = z.object({
  id: z.number(),
  user_id: z.number(),
  name: z.string(),
  value: z.string().optional(),
  private: z.boolean(),
  description: z.string().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

export const UserSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Name is required'),
  username: z.string().optional(),
  first_name: z.string().min(1, 'First name is required'),
  last_name: z.string().min(1, 'Last name is required'),
  email: z.string().email('Please enter a valid email address'),
  is_active: z.boolean(),
  phone: z.string().optional(),
  security_question: z.string().optional(),
  security_answer: z.string().optional(),
  confirmed_email: z.boolean().optional(),
  oauth_provider: z.string().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
  role_id: z.number().optional(),
  role_by_role_id: UserRoleSchema.optional(),
  user_app_role_by_user_id: z.array(UserAppRoleSchema).optional(),
  user_lookup_by_user_id: z.array(UserLookupSchema).optional(),
  adldap: z.string().optional(),
  saml: z.string().optional(),
});

// System Schemas
export const SystemInfoSchema = z.object({
  platform: z.object({
    name: z.string(),
    version: z.string(),
    build_date: z.string().optional(),
    commit_hash: z.string().optional(),
  }),
  server_os: z.object({
    name: z.string(),
    version: z.string(),
    architecture: z.string(),
  }),
  php: z.object({
    version: z.string(),
    extensions: z.array(z.string()),
  }),
  database: z.object({
    driver: z.string(),
    version: z.string(),
  }),
  cache: z.object({
    driver: z.string(),
  }),
}).catchall(z.unknown());

// Configuration Schemas
export const EmailTemplateSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Template name is required'),
  description: z.string().optional(),
  to: z.array(z.string().email()),
  cc: z.array(z.string().email()).optional(),
  bcc: z.array(z.string().email()).optional(),
  subject: z.string().min(1, 'Subject is required'),
  body_text: z.string().optional(),
  body_html: z.string().optional(),
  from_name: z.string().optional(),
  from_email: z.string().email().optional(),
  reply_to_name: z.string().optional(),
  reply_to_email: z.string().email().optional(),
  defaults: z.array(z.unknown()).optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

export const LookupKeySchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Key name is required'),
  value: z.string().optional(),
  private: z.boolean(),
  description: z.string().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

export const CorsConfigSchema = z.object({
  id: z.number(),
  path: z.string().min(1, 'Path is required'),
  origin: z.string().min(1, 'Origin is required'),
  header: z.string(),
  method: z.string(),
  max_age: z.number().optional(),
  supports_credentials: z.boolean().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

// Event & Script Schemas
export const EventSchema = z.object({
  name: z.string(),
  label: z.string(),
  description: z.string().optional(),
  service_name: z.string().optional(),
  resource: z.string().optional(),
  verb: z.string().optional(),
});

export const EventScriptSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Script name is required'),
  type: z.string(),
  is_active: z.boolean(),
  source: z.string().min(1, 'Script source is required'),
  config: z.unknown().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

export const ScriptTypeSchema = z.object({
  name: z.string(),
  label: z.string(),
  description: z.string().optional(),
  sandboxed: z.boolean().optional(),
  extensions: z.array(z.string()).optional(),
});

// Scheduler Schemas
export const SchedulerTaskSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Task name is required'),
  description: z.string().optional(),
  is_active: z.boolean(),
  verb: z.string().min(1, 'HTTP verb is required'),
  service: z.string().min(1, 'Service is required'),
  resource: z.string().optional(),
  payload: z.unknown().optional(),
  frequency: z.number().min(1, 'Frequency must be at least 1'),
  frequency_type: z.enum(['minute', 'hour', 'day', 'week', 'month']),
  start_date: z.string(),
  end_date: z.string().optional(),
  last_run: z.string().optional(),
  next_run: z.string().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

// Limit Schemas
export const LimitSchema = z.object({
  id: z.number(),
  name: z.string().min(1, 'Limit name is required'),
  description: z.string().optional(),
  is_active: z.boolean(),
  type: z.enum(['user', 'role', 'service', 'each_user']),
  key_text: z.string().min(1, 'Key text is required'),
  rate: z.number().min(1, 'Rate must be at least 1'),
  period: z.enum(['minute', 'hour', 'day', 'week', 'month']),
  user_id: z.number().optional(),
  role_id: z.number().optional(),
  service_id: z.number().optional(),
  verb_mask: z.number().optional(),
  created_date: z.string(),
  last_modified_date: z.string(),
});

// Files Schemas
export const FileInfoSchema = z.object({
  name: z.string(),
  path: z.string(),
  type: z.enum(['file', 'folder']),
  size: z.number().optional(),
  last_modified: z.string().optional(),
  content_type: z.string().optional(),
});

// API Docs Schemas
export const ApiDocSchema = z.object({
  service_name: z.string(),
  paths: z.record(z.unknown()),
  definitions: z.record(z.unknown()).optional(),
  info: z.object({
    title: z.string(),
    description: z.string().optional(),
    version: z.string(),
  }).optional(),
});

// Request Options Schema
export const RequestOptionsSchema = z.object({
  limit: z.number().optional(),
  offset: z.number().optional(),
  filter: z.string().optional(),
  sort: z.string().optional(),
  fields: z.string().optional(),
  related: z.string().optional(),
  include_count: z.boolean().optional(),
  refresh: z.boolean().optional(),
  additionalParams: z.record(z.unknown()).optional(),
  additionalHeaders: z.record(z.string()).optional(),
  contentType: z.string().optional(),
  showSpinner: z.boolean().optional(),
  snackbarSuccess: z.string().optional(),
  snackbarError: z.string().optional(),
  includeCacheControl: z.boolean().optional(),
});

// List Response Schemas
export const createListResponseSchema = <T extends z.ZodType>(itemSchema: T) =>
  z.object({
    resource: z.array(itemSchema),
    meta: z.object({
      count: z.number(),
      schema: z.array(z.unknown()).optional(),
    }),
  });

// Export commonly used list schemas
export const UserListResponseSchema = createListResponseSchema(UserSchema);
export const RoleListResponseSchema = createListResponseSchema(RoleSchema);
export const AppListResponseSchema = createListResponseSchema(AppSchema);
export const ServiceListResponseSchema = createListResponseSchema(ServiceSchema);
export const EmailTemplateListResponseSchema = createListResponseSchema(EmailTemplateSchema);
export const EventScriptListResponseSchema = createListResponseSchema(EventScriptSchema);
export const SchedulerTaskListResponseSchema = createListResponseSchema(SchedulerTaskSchema);
export const LimitListResponseSchema = createListResponseSchema(LimitSchema);
export const FileListResponseSchema = createListResponseSchema(FileInfoSchema);

// Type exports for use in components
export type LoginCredentials = z.infer<typeof LoginCredentialsSchema>;
export type RegisterDetails = z.infer<typeof RegisterDetailsSchema>;
export type UserSession = z.infer<typeof UserSessionSchema>;
export type User = z.infer<typeof UserSchema>;
export type Role = z.infer<typeof RoleSchema>;
export type App = z.infer<typeof AppSchema>;
export type Service = z.infer<typeof ServiceSchema>;
export type EmailTemplate = z.infer<typeof EmailTemplateSchema>;
export type EventScript = z.infer<typeof EventScriptSchema>;
export type SchedulerTask = z.infer<typeof SchedulerTaskSchema>;
export type Limit = z.infer<typeof LimitSchema>;
export type RequestOptions = z.infer<typeof RequestOptionsSchema>;