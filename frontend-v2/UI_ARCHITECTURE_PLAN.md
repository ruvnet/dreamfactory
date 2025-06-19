# DreamFactory UI Architecture Plan

## Tech Stack
- **Framework**: React 19.1.0 with TypeScript
- **Routing**: React Router v7
- **State Management**: Zustand with persistence
- **Styling**: Tailwind CSS v4 with Headless UI
- **Data Fetching**: TanStack Query (React Query)
- **Form Handling**: React Hook Form with Zod validation
- **Icons**: Heroicons
- **Build Tool**: Vite
- **Testing**: Vitest + Playwright

## Navigation Structure

### Primary Navigation (Sidebar)
```
Dashboard
├── Overview (/)
├── Analytics (/analytics)
└── Activity (/activity)

API Services
├── All Services (/services)
├── Create Service (/services/create)
├── Service Details (/services/:id)
└── API Docs (/services/:id/docs)

Users & Roles
├── Users (/users)
├── User Details (/users/:id)
├── Roles (/roles)
├── Role Details (/roles/:id)
└── Permissions (/permissions)

Apps
├── All Apps (/apps)
├── Create App (/apps/create)
├── App Details (/apps/:id)
└── App Keys (/apps/:id/keys)

Data
├── Schema (/data/schema)
├── Data Browser (/data/browser)
├── Import/Export (/data/transfer)
└── Backups (/data/backups)

Files
├── File Manager (/files)
├── Storage Services (/files/storage)
└── File Policies (/files/policies)

Email
├── Templates (/email/templates)
├── Configuration (/email/config)
└── Logs (/email/logs)

System
├── Configuration (/system/config)
├── Cache (/system/cache)
├── Scheduler (/system/scheduler)
├── Event Logs (/system/logs)
└── API Limits (/system/limits)
```

### Secondary Navigation (Top Bar)
- Search (global search)
- Notifications dropdown
- User menu
  - Profile (/profile)
  - Settings (/settings)
  - Logout

## Component Hierarchy

### Layout Components
```
AppRoot
├── AuthProvider
├── QueryClientProvider
├── ThemeProvider
└── RouterProvider
    ├── PublicRoute
    │   └── AuthLayout
    │       └── [Login|Register|ForgotPassword]
    └── ProtectedRoute
        └── DashboardLayout
            ├── Sidebar
            │   ├── Logo
            │   ├── Navigation
            │   └── CollapseToggle
            ├── Header
            │   ├── SearchBar
            │   ├── NotificationCenter
            │   └── UserMenu
            └── MainContent
                └── [PageComponent]
```

### Component Structure
```
src/
├── components/
│   ├── atoms/           # Basic UI elements
│   │   ├── Button/
│   │   ├── Input/
│   │   ├── Select/
│   │   ├── Checkbox/
│   │   ├── Radio/
│   │   ├── Toggle/
│   │   ├── Badge/
│   │   ├── Avatar/
│   │   ├── Icon/
│   │   ├── Spinner/
│   │   └── Tooltip/
│   ├── molecules/       # Composite components
│   │   ├── FormField/
│   │   ├── DataTable/
│   │   ├── Card/
│   │   ├── Modal/
│   │   ├── Dropdown/
│   │   ├── Tabs/
│   │   ├── Breadcrumb/
│   │   ├── Pagination/
│   │   └── SearchInput/
│   ├── organisms/       # Complex components
│   │   ├── Sidebar/
│   │   ├── Header/
│   │   ├── ServiceCard/
│   │   ├── UserTable/
│   │   ├── ApiDocViewer/
│   │   ├── FileUploader/
│   │   └── NotificationList/
│   └── templates/       # Page layouts
│       ├── AuthLayout/
│       ├── DashboardLayout/
│       └── ErrorLayout/
```

## Routing Structure

### Route Configuration
```typescript
const routes = {
  public: {
    login: '/login',
    register: '/register',
    forgotPassword: '/forgot-password',
    resetPassword: '/reset-password/:token',
  },
  protected: {
    dashboard: {
      index: '/',
      analytics: '/analytics',
      activity: '/activity',
    },
    services: {
      list: '/services',
      create: '/services/create',
      detail: '/services/:id',
      edit: '/services/:id/edit',
      docs: '/services/:id/docs',
    },
    users: {
      list: '/users',
      create: '/users/create',
      detail: '/users/:id',
      edit: '/users/:id/edit',
    },
    roles: {
      list: '/roles',
      create: '/roles/create',
      detail: '/roles/:id',
      edit: '/roles/:id/edit',
    },
    apps: {
      list: '/apps',
      create: '/apps/create',
      detail: '/apps/:id',
      edit: '/apps/:id/edit',
      keys: '/apps/:id/keys',
    },
    data: {
      schema: '/data/schema',
      browser: '/data/browser',
      transfer: '/data/transfer',
      backups: '/data/backups',
    },
    files: {
      manager: '/files',
      storage: '/files/storage',
      policies: '/files/policies',
    },
    email: {
      templates: '/email/templates',
      config: '/email/config',
      logs: '/email/logs',
    },
    system: {
      config: '/system/config',
      cache: '/system/cache',
      scheduler: '/system/scheduler',
      logs: '/system/logs',
      limits: '/system/limits',
    },
    profile: '/profile',
    settings: '/settings',
  },
}
```

## State Management Strategy

### Store Architecture
```typescript
// Auth Store
interface AuthStore {
  user: User | null
  token: string | null
  refreshToken: string | null
  isAuthenticated: boolean
  permissions: Permission[]
  login: (credentials: LoginCredentials) => Promise<void>
  logout: () => void
  refreshSession: () => Promise<void>
  hasPermission: (permission: string) => boolean
}

// UI Store
interface UIStore {
  theme: 'light' | 'dark' | 'system'
  sidebarOpen: boolean
  sidebarCollapsed: boolean
  mobileMenuOpen: boolean
  notifications: Notification[]
  toasts: Toast[]
  modals: Modal[]
  globalSearch: string
  setTheme: (theme: Theme) => void
  toggleSidebar: () => void
  addNotification: (notification: Notification) => void
  showToast: (toast: ToastOptions) => void
  openModal: (modal: ModalOptions) => void
}

// Services Store
interface ServicesStore {
  services: Service[]
  selectedService: Service | null
  isLoading: boolean
  error: string | null
  fetchServices: () => Promise<void>
  createService: (data: ServiceData) => Promise<void>
  updateService: (id: string, data: ServiceData) => Promise<void>
  deleteService: (id: string) => Promise<void>
}

// App Config Store
interface AppConfigStore {
  config: AppConfig
  features: FeatureFlags
  isLoading: boolean
  fetchConfig: () => Promise<void>
  updateConfig: (config: Partial<AppConfig>) => Promise<void>
}
```

## Authentication Flow

### Login Process
1. User enters credentials on login page
2. Submit to `/api/auth/login`
3. Receive JWT token and refresh token
4. Store tokens in AuthStore (persisted)
5. Fetch user profile and permissions
6. Redirect to dashboard

### Session Management
- JWT token expiry: 15 minutes
- Refresh token expiry: 7 days
- Auto-refresh before token expiry
- Logout clears all persisted data
- Protected routes check authentication

### Permission System
```typescript
interface Permission {
  resource: string
  action: string
  scope?: 'own' | 'all'
}

// Route Guards
const ProtectedRoute = ({ 
  children, 
  permission 
}: { 
  children: ReactNode
  permission?: string 
}) => {
  const { isAuthenticated, hasPermission } = useAuthStore()
  
  if (!isAuthenticated) {
    return <Navigate to="/login" />
  }
  
  if (permission && !hasPermission(permission)) {
    return <Navigate to="/403" />
  }
  
  return children
}
```

## Responsive Design Breakpoints

### Breakpoint System
```css
/* Mobile First Approach */
sm: 640px   /* Small tablets */
md: 768px   /* Tablets */
lg: 1024px  /* Desktop */
xl: 1280px  /* Large screens */
2xl: 1536px /* Extra large screens */
```

### Layout Behavior
- **Mobile (< 768px)**
  - Sidebar: Hidden, toggle with hamburger menu
  - Navigation: Full screen overlay
  - Tables: Card view or horizontal scroll
  - Forms: Single column

- **Tablet (768px - 1024px)**
  - Sidebar: Collapsible to icon-only
  - Navigation: Persistent but condensed
  - Tables: Responsive with priority columns
  - Forms: Adaptive columns

- **Desktop (> 1024px)**
  - Sidebar: Full width, collapsible
  - Navigation: Full featured
  - Tables: Full data display
  - Forms: Multi-column layouts

## Design System

### Color Palette
```typescript
const colors = {
  primary: {
    50: '#eff6ff',
    500: '#3b82f6',
    900: '#1e3a8a',
  },
  gray: {
    50: '#f9fafb',
    500: '#6b7280',
    900: '#111827',
  },
  success: '#10b981',
  warning: '#f59e0b',
  error: '#ef4444',
  info: '#3b82f6',
}
```

### Typography Scale
```typescript
const typography = {
  xs: '0.75rem',
  sm: '0.875rem',
  base: '1rem',
  lg: '1.125rem',
  xl: '1.25rem',
  '2xl': '1.5rem',
  '3xl': '1.875rem',
  '4xl': '2.25rem',
}
```

### Spacing System
```typescript
const spacing = {
  0: '0',
  1: '0.25rem',
  2: '0.5rem',
  3: '0.75rem',
  4: '1rem',
  6: '1.5rem',
  8: '2rem',
  12: '3rem',
  16: '4rem',
}
```

## Performance Optimizations

### Code Splitting
- Route-based code splitting
- Lazy load heavy components
- Dynamic imports for modals
- Vendor chunk optimization

### Data Management
- React Query for caching
- Optimistic updates
- Background refetching
- Pagination for large datasets
- Virtual scrolling for lists

### Asset Optimization
- Image lazy loading
- SVG sprite sheets
- Font subsetting
- CSS purging with Tailwind

## Accessibility Features

### WCAG 2.1 AA Compliance
- Semantic HTML structure
- ARIA labels and roles
- Keyboard navigation
- Focus management
- Screen reader support
- Color contrast ratios
- Reduced motion support

### Keyboard Shortcuts
- `Ctrl/Cmd + K`: Global search
- `Ctrl/Cmd + /`: Toggle sidebar
- `Esc`: Close modals
- Tab navigation throughout

## Security Considerations

### Frontend Security
- XSS prevention with React
- CSRF token handling
- Secure cookie flags
- Content Security Policy
- Input sanitization
- API request signing

### Authentication Security
- Secure token storage
- Auto-logout on inactivity
- Session invalidation
- Password strength requirements
- 2FA support ready

This architecture provides a solid foundation for building a modern, scalable, and user-friendly admin interface for DreamFactory.