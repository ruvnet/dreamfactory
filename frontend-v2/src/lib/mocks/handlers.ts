import { http, HttpResponse } from 'msw';
import { API_ENDPOINTS } from '../constants/api';
import type {
  User,
  UserSession,
  Service,
  ServiceType,
  Role,
  App,
  LoginCredentials,
  APIListResponse,
} from '../types/api';

// Mock data
const mockUsers: User[] = [
  {
    id: 1,
    name: 'John Doe',
    first_name: 'John',
    last_name: 'Doe',
    email: 'john.doe@example.com',
    is_active: true,
    phone: '+1234567890',
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
    role_id: 1,
    confirmed_email: true,
  },
  {
    id: 2,
    name: 'Jane Smith',
    first_name: 'Jane',
    last_name: 'Smith',
    email: 'jane.smith@example.com',
    is_active: true,
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
    role_id: 2,
    confirmed_email: true,
  },
];

const mockRoles: Role[] = [
  {
    id: 1,
    name: 'Admin',
    description: 'System Administrator',
    is_active: true,
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
  },
  {
    id: 2,
    name: 'User',
    description: 'Regular User',
    is_active: true,
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
  },
];

const mockServices: Service[] = [
  {
    id: 1,
    name: 'db',
    label: 'Database Service',
    description: 'Default database service',
    is_active: true,
    type: 'sql_db',
    mutable: false,
    deletable: false,
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
  },
  {
    id: 2,
    name: 'files',
    label: 'File Service',
    description: 'Local file storage service',
    is_active: true,
    type: 'local_file',
    mutable: true,
    deletable: true,
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
  },
];

const mockServiceTypes: ServiceType[] = [
  {
    name: 'sql_db',
    label: 'SQL Database',
    description: 'SQL Database Service',
    group: 'Database',
    subscription_required: false,
  },
  {
    name: 'local_file',
    label: 'Local File Storage',
    description: 'Local File Storage Service',
    group: 'File',
    subscription_required: false,
  },
  {
    name: 'aws_s3',
    label: 'Amazon S3',
    description: 'Amazon S3 File Storage Service',
    group: 'File',
    subscription_required: true,
  },
];

const mockApps: App[] = [
  {
    id: 1,
    name: 'admin',
    description: 'DreamFactory Admin Console',
    is_active: true,
    url: '/admin',
    is_url_external: false,
    api_key: 'admin-api-key-123',
    role_id: 1,
    created_date: '2024-01-01T00:00:00Z',
    last_modified_date: '2024-01-01T00:00:00Z',
  },
];

// Mock session user
const mockSessionUser: UserSession = {
  id: 1,
  name: 'John Doe',
  first_name: 'John',
  last_name: 'Doe',
  email: 'john.doe@example.com',
  is_active: true,
  created_date: '2024-01-01T00:00:00Z',
  last_modified_date: '2024-01-01T00:00:00Z',
  session_token: 'mock-session-token-123',
  session_id: 'mock-session-id-123',
  role: mockRoles[0],
  isSysAdmin: true,
  confirmed_email: true,
};

// Helper functions
const createListResponse = <T>(items: T[], count?: number): APIListResponse<T> => ({
  resource: items,
  meta: {
    count: count ?? items.length,
  },
});

const findById = <T extends { id: number }>(items: T[], id: string | number) => {
  const numId = typeof id === 'string' ? parseInt(id) : id;
  return items.find(item => item.id === numId);
};

const removeById = <T extends { id: number }>(items: T[], id: string | number) => {
  const numId = typeof id === 'string' ? parseInt(id) : id;
  return items.filter(item => item.id !== numId);
};

// Request handlers
export const handlers = [
  // Authentication endpoints
  http.post(API_ENDPOINTS.USER_SESSION, async ({ request }) => {
    const credentials = await request.json() as LoginCredentials;
    
    // Simple validation
    if (credentials.email === 'john.doe@example.com' && credentials.password === 'password') {
      return HttpResponse.json(mockSessionUser);
    }
    
    return new HttpResponse(
      JSON.stringify({
        error: {
          code: 401,
          message: 'Invalid credentials',
          status_code: 401,
        },
      }),
      { status: 401 }
    );
  }),

  http.get(API_ENDPOINTS.USER_SESSION, () => {
    return HttpResponse.json(mockSessionUser);
  }),

  http.delete(API_ENDPOINTS.USER_SESSION, () => {
    return HttpResponse.json({ success: true });
  }),

  http.post(API_ENDPOINTS.REGISTER, async ({ request }) => {
    const data = await request.json();
    return HttpResponse.json({ success: true });
  }),

  // Users endpoints
  http.get(API_ENDPOINTS.SYSTEM_USER, ({ request }) => {
    const url = new URL(request.url);
    const limit = parseInt(url.searchParams.get('limit') || '50');
    const offset = parseInt(url.searchParams.get('offset') || '0');
    
    const paginatedUsers = mockUsers.slice(offset, offset + limit);
    return HttpResponse.json(createListResponse(paginatedUsers, mockUsers.length));
  }),

  http.get(`${API_ENDPOINTS.SYSTEM_USER}/:id`, ({ params }) => {
    const user = findById(mockUsers, params.id as string);
    if (!user) {
      return new HttpResponse(
        JSON.stringify({
          error: {
            code: 404,
            message: 'User not found',
            status_code: 404,
          },
        }),
        { status: 404 }
      );
    }
    return HttpResponse.json(user);
  }),

  http.post(API_ENDPOINTS.SYSTEM_USER, async ({ request }) => {
    const data = await request.json() as Partial<User>;
    const newUser: User = {
      id: mockUsers.length + 1,
      name: data.name || '',
      first_name: data.first_name || '',
      last_name: data.last_name || '',
      email: data.email || '',
      is_active: data.is_active ?? true,
      created_date: new Date().toISOString(),
      last_modified_date: new Date().toISOString(),
      role_id: data.role_id,
      confirmed_email: true,
    };
    mockUsers.push(newUser);
    return HttpResponse.json(newUser);
  }),

  http.put(`${API_ENDPOINTS.SYSTEM_USER}/:id`, async ({ params, request }) => {
    const data = await request.json() as Partial<User>;
    const userIndex = mockUsers.findIndex(u => u.id === parseInt(params.id as string));
    
    if (userIndex === -1) {
      return new HttpResponse(
        JSON.stringify({
          error: {
            code: 404,
            message: 'User not found',
            status_code: 404,
          },
        }),
        { status: 404 }
      );
    }
    
    mockUsers[userIndex] = {
      ...mockUsers[userIndex],
      ...data,
      last_modified_date: new Date().toISOString(),
    };
    
    return HttpResponse.json(mockUsers[userIndex]);
  }),

  http.delete(`${API_ENDPOINTS.SYSTEM_USER}/:id`, ({ params }) => {
    const userId = parseInt(params.id as string);
    const userIndex = mockUsers.findIndex(u => u.id === userId);
    
    if (userIndex === -1) {
      return new HttpResponse(
        JSON.stringify({
          error: {
            code: 404,
            message: 'User not found',
            status_code: 404,
          },
        }),
        { status: 404 }
      );
    }
    
    mockUsers.splice(userIndex, 1);
    return HttpResponse.json({ success: true });
  }),

  // Profile endpoints
  http.get(API_ENDPOINTS.USER_PROFILE, () => {
    return HttpResponse.json(mockUsers[0]);
  }),

  http.put(API_ENDPOINTS.USER_PROFILE, async ({ request }) => {
    const data = await request.json() as Partial<User>;
    const updatedUser = {
      ...mockUsers[0],
      ...data,
      last_modified_date: new Date().toISOString(),
    };
    mockUsers[0] = updatedUser;
    return HttpResponse.json(updatedUser);
  }),

  // Services endpoints
  http.get(API_ENDPOINTS.SYSTEM_SERVICE, ({ request }) => {
    const url = new URL(request.url);
    const limit = parseInt(url.searchParams.get('limit') || '50');
    const offset = parseInt(url.searchParams.get('offset') || '0');
    
    const paginatedServices = mockServices.slice(offset, offset + limit);
    return HttpResponse.json(createListResponse(paginatedServices, mockServices.length));
  }),

  http.get(`${API_ENDPOINTS.SYSTEM_SERVICE}/:id`, ({ params }) => {
    const service = findById(mockServices, params.id as string);
    if (!service) {
      return new HttpResponse(
        JSON.stringify({
          error: {
            code: 404,
            message: 'Service not found',
            status_code: 404,
          },
        }),
        { status: 404 }
      );
    }
    return HttpResponse.json(service);
  }),

  http.post(API_ENDPOINTS.SYSTEM_SERVICE, async ({ request }) => {
    const data = await request.json() as Partial<Service>;
    const newService: Service = {
      id: mockServices.length + 1,
      name: data.name || '',
      label: data.label || '',
      description: data.description || '',
      is_active: data.is_active ?? true,
      type: data.type || '',
      mutable: data.mutable ?? true,
      deletable: data.deletable ?? true,
      created_date: new Date().toISOString(),
      last_modified_date: new Date().toISOString(),
    };
    mockServices.push(newService);
    return HttpResponse.json(newService);
  }),

  // Service types endpoint
  http.get(API_ENDPOINTS.SERVICE_TYPE, () => {
    return HttpResponse.json(createListResponse(mockServiceTypes));
  }),

  // Roles endpoints
  http.get(API_ENDPOINTS.ROLES, ({ request }) => {
    const url = new URL(request.url);
    const limit = parseInt(url.searchParams.get('limit') || '50');
    const offset = parseInt(url.searchParams.get('offset') || '0');
    
    const paginatedRoles = mockRoles.slice(offset, offset + limit);
    return HttpResponse.json(createListResponse(paginatedRoles, mockRoles.length));
  }),

  http.get(`${API_ENDPOINTS.ROLES}/:id`, ({ params }) => {
    const role = findById(mockRoles, params.id as string);
    if (!role) {
      return new HttpResponse(
        JSON.stringify({
          error: {
            code: 404,
            message: 'Role not found',
            status_code: 404,
          },
        }),
        { status: 404 }
      );
    }
    return HttpResponse.json(role);
  }),

  // Apps endpoints
  http.get(API_ENDPOINTS.APP, ({ request }) => {
    const url = new URL(request.url);
    const limit = parseInt(url.searchParams.get('limit') || '50');
    const offset = parseInt(url.searchParams.get('offset') || '0');
    
    const paginatedApps = mockApps.slice(offset, offset + limit);
    return HttpResponse.json(createListResponse(paginatedApps, mockApps.length));
  }),

  http.get(`${API_ENDPOINTS.APP}/:id`, ({ params }) => {
    const app = findById(mockApps, params.id as string);
    if (!app) {
      return new HttpResponse(
        JSON.stringify({
          error: {
            code: 404,
            message: 'App not found',
            status_code: 404,
          },
        }),
        { status: 404 }
      );
    }
    return HttpResponse.json(app);
  }),

  // System info endpoint
  http.get(API_ENDPOINTS.SYSTEM, () => {
    return HttpResponse.json({
      platform: {
        name: 'DreamFactory',
        version: '7.1.0',
        build_date: '2024-01-01',
        commit_hash: 'abc123',
      },
      server_os: {
        name: 'Linux',
        version: '5.4.0',
        architecture: 'x86_64',
      },
      php: {
        version: '8.1.0',
        extensions: ['json', 'mbstring', 'openssl'],
      },
      database: {
        driver: 'mysql',
        version: '8.0.0',
      },
      cache: {
        driver: 'redis',
      },
    });
  }),

  // API Docs endpoint
  http.get(`${API_ENDPOINTS.API_DOCS}/:service`, ({ params }) => {
    return HttpResponse.json({
      service_name: params.service,
      paths: {
        '/': {
          get: {
            summary: 'Get service information',
            responses: {
              '200': {
                description: 'Success',
              },
            },
          },
        },
      },
      info: {
        title: `${params.service} API`,
        description: `API documentation for ${params.service} service`,
        version: '1.0.0',
      },
    });
  }),

  // Files endpoint
  http.get(API_ENDPOINTS.FILES, ({ request }) => {
    const url = new URL(request.url);
    const path = url.searchParams.get('path') || '';
    
    return HttpResponse.json(createListResponse([
      {
        name: 'folder1',
        path: `${path}/folder1`,
        type: 'folder' as const,
        last_modified: '2024-01-01T00:00:00Z',
      },
      {
        name: 'file1.txt',
        path: `${path}/file1.txt`,
        type: 'file' as const,
        size: 1024,
        last_modified: '2024-01-01T00:00:00Z',
        content_type: 'text/plain',
      },
    ]));
  }),

  // Fallback handler for unmatched requests
  http.all('*', ({ request }) => {
    console.warn(`Unhandled ${request.method} request to ${request.url}`);
    return new HttpResponse(
      JSON.stringify({
        error: {
          code: 404,
          message: 'Endpoint not found',
          status_code: 404,
        },
      }),
      { status: 404 }
    );
  }),
];

export default handlers;