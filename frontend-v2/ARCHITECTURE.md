# DreamFactory Frontend V2 Architecture

## Project Structure

This project follows atomic design principles combined with modern React architecture patterns for maximum scalability and maintainability.

```
src/
├── components/           # UI Components (Atomic Design)
│   ├── atoms/           # Basic building blocks
│   │   ├── Button/      # Reusable button components
│   │   ├── Input/       # Form input components
│   │   ├── Icon/        # Icon components
│   │   ├── Badge/       # Status/notification badges
│   │   ├── Avatar/      # User avatar components
│   │   └── Spinner/     # Loading spinners
│   ├── molecules/       # Groups of atoms
│   │   ├── Card/        # Card container components
│   │   ├── Modal/       # Modal dialog components
│   │   ├── Dropdown/    # Dropdown menu components
│   │   ├── Navigation/  # Navigation components
│   │   └── Form/        # Form field groups
│   ├── organisms/       # Complex UI sections
│   │   ├── Header/      # Application header
│   │   ├── Sidebar/     # Navigation sidebar
│   │   ├── DataTable/   # Data table components
│   │   └── Dashboard/   # Dashboard sections
│   └── templates/       # Page layout structures
│       ├── Layout/      # Base layout template
│       ├── AuthLayout/  # Authentication pages layout
│       └── DashboardLayout/ # Dashboard layout
├── pages/               # Route components
│   ├── auth/           # Authentication pages
│   ├── dashboard/      # Dashboard pages
│   ├── services/       # Service management pages
│   ├── users/          # User management pages
│   ├── roles/          # Role management pages
│   ├── settings/       # Application settings
│   └── profile/        # User profile pages
├── hooks/              # Custom React hooks
├── utils/              # Utility functions
├── services/           # API service functions
├── stores/             # State management (Zustand)
├── types/              # TypeScript type definitions
├── lib/                # Library configurations
├── assets/             # Static assets
├── context/            # React context providers
└── test/               # Test utilities and fixtures
```

## Design System Architecture

### Atomic Design Principles

**Atoms** - The basic building blocks of matter. In our case, HTML elements like buttons, inputs, and icons.

**Molecules** - Groups of atoms bonded together to form the smallest fundamental units of compounds. Examples: search form, card header.

**Organisms** - Groups of molecules joined together to form relatively complex, distinct sections. Examples: header, sidebar, data table.

**Templates** - Page-level objects that place components into a layout and articulate the design's underlying content structure.

**Pages** - Specific instances of templates that show what a UI looks like with real representative content.

## Component Structure

Each component follows this structure:

```
ComponentName/
├── index.ts           # Barrel export
├── ComponentName.tsx  # Main component
├── ComponentName.test.tsx # Unit tests
├── ComponentName.stories.tsx # Storybook stories
└── types.ts          # Component-specific types
```

## Naming Conventions

### Files and Folders
- **Components**: PascalCase (e.g., `Button`, `DataTable`)
- **Pages**: kebab-case (e.g., `user-profile`, `api-settings`)
- **Utilities**: camelCase (e.g., `formatDate`, `apiClient`)
- **Types**: PascalCase with descriptive suffix (e.g., `UserType`, `ApiResponseType`)

### Code
- **Variables/Functions**: camelCase
- **Constants**: SCREAMING_SNAKE_CASE
- **Interfaces/Types**: PascalCase with descriptive suffix
- **Components**: PascalCase

## State Management

### Zustand Stores
- **AuthStore**: User authentication state
- **UIStore**: Global UI state (theme, modals, notifications)
- **DataStore**: Application data cache
- **SettingsStore**: User preferences and app settings

### React Query
- Server state management
- Caching and synchronization
- Background updates
- Optimistic updates

## API Architecture

### Service Layer
```
services/
├── auth.service.ts    # Authentication APIs
├── user.service.ts    # User management APIs
├── role.service.ts    # Role management APIs
└── api.service.ts     # Generic API utilities
```

### API Client Configuration
- Axios-based HTTP client
- Request/response interceptors
- Error handling
- Token management
- Type-safe endpoints

## Testing Strategy

### Unit Tests (Vitest + React Testing Library)
- Component behavior testing
- Hook testing  
- Utility function testing
- 100% coverage target

### Integration Tests
- API service testing
- State management testing
- Component integration testing

### E2E Tests (Playwright)
- User workflow testing
- Cross-browser compatibility
- Performance testing
- Accessibility testing

## Development Workflow

### Code Quality
- ESLint for code linting
- Prettier for code formatting
- TypeScript for type safety
- Husky for pre-commit hooks

### Component Development
1. Write Storybook story first
2. Implement component with TypeScript
3. Write unit tests
4. Test in isolation via Storybook
5. Integrate into pages

### Performance Optimization
- Code splitting with React.lazy()
- Bundle optimization with Vite
- Image optimization
- Tree shaking
- Lazy loading

## Deployment

### Build Process
- TypeScript compilation
- Vite production build
- Asset optimization
- Bundle analysis

### CI/CD Pipeline
- Automated testing
- Build verification
- Deployment to staging
- Production deployment

This architecture ensures scalability, maintainability, and optimal developer experience while maintaining high performance and code quality standards.