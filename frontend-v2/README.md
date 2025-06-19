# DreamFactory Frontend V2

A modern, enterprise-grade frontend application built with React 19, TypeScript, and Vite for the DreamFactory API management platform.

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Start Storybook for component development
npm run storybook
```

## 🛠 Tech Stack

- **Framework:** React 19.1.0 with TypeScript 5.8.3
- **Build Tool:** Vite 6.3.5
- **Styling:** Tailwind CSS 4.1.10 + Headless UI
- **State Management:** Zustand + React Query
- **Routing:** React Router 7.6.2
- **Testing:** Vitest + React Testing Library + Playwright
- **Component Development:** Storybook 9.0.12
- **Code Quality:** ESLint + Prettier

## 📁 Project Structure

```
src/
├── components/          # Atomic design components
│   ├── atoms/          # Basic UI elements (Button, Input, etc.)
│   ├── molecules/      # Component combinations (Card, Modal, etc.)
│   ├── organisms/      # Complex UI sections (Header, DataTable, etc.)
│   └── templates/      # Page layouts
├── pages/              # Route components
├── hooks/              # Custom React hooks
├── stores/             # Zustand state stores
├── services/           # API service functions
├── utils/              # Utility functions
├── types/              # TypeScript definitions
├── lib/                # Library configurations
└── test/               # Test utilities
```

## 🧪 Testing

```bash
# Run all tests
npm run test:all

# Unit tests with coverage
npm run test:coverage

# E2E tests
npm run test:e2e

# E2E tests with UI
npm run test:e2e:ui
```

## 📦 Scripts

```bash
# Development
npm run dev              # Start dev server
npm run storybook        # Start Storybook

# Building
npm run build            # Production build
npm run preview          # Preview production build

# Code Quality
npm run typecheck        # TypeScript type checking
npm run lint             # ESLint
npm run lint:fix         # Fix ESLint issues
npm run format           # Format code with Prettier
npm run format:check     # Check code formatting

# Testing
npm run test             # Run tests in watch mode
npm run test:run         # Run tests once
npm run test:coverage    # Run tests with coverage
npm run test:e2e         # Run E2E tests
npm run test:all         # Run all tests
```

## 🎨 Component Development

This project follows atomic design principles:

1. **Atoms** - Basic UI elements (Button, Input, Icon)
2. **Molecules** - Groups of atoms (Card, Modal, Form fields)
3. **Organisms** - Complex UI sections (Header, DataTable, Dashboard)
4. **Templates** - Page layout structures
5. **Pages** - Complete page implementations

### Creating a New Component

1. Create component directory in appropriate atomic level
2. Implement component with TypeScript
3. Add Storybook story
4. Write unit tests
5. Export from index file

Example Button component structure:
```
components/atoms/Button/
├── index.ts
├── Button.tsx
├── Button.stories.tsx
└── Button.test.tsx
```

## 🔧 Configuration

### TypeScript
- Strict mode enabled
- Path mapping configured (`@/*` aliases)
- Type-only imports enforced

### ESLint
- React and TypeScript rules
- Prettier integration
- Custom rules for code organization

### Tailwind CSS
- Custom design tokens
- Component classes
- Responsive utilities
- Dark mode support

## 🔐 Authentication & Routing

- JWT-based authentication
- Route guards for protected pages
- Lazy loading for performance
- Nested routing with layouts

## 📊 State Management

### Client State (Zustand)
- `useAuthStore` - Authentication state
- `useUIStore` - UI state (theme, modals, toasts)

### Server State (React Query)
- API data caching
- Background updates
- Optimistic updates
- Query invalidation

## 🎯 Performance

- Code splitting with React.lazy()
- Bundle optimization with Vite
- Tree shaking for unused code
- Optimized build output

## 🧑‍💻 Development Guidelines

### Code Style
- Use TypeScript for all new code
- Follow atomic design principles
- Prefer functional components with hooks
- Use Tailwind for styling

### Testing Requirements
- Write tests for all components
- Test user interactions
- Mock external dependencies
- Maintain high test coverage

### Performance Considerations
- Lazy load routes and components
- Optimize bundle size
- Use React.memo for expensive renders
- Implement proper loading states

## 🚀 Deployment

The project includes GitHub Actions workflows for:
- Code quality checks
- Automated testing
- Security scanning
- Performance testing with Lighthouse
- Automated deployment

## 📖 Architecture Documentation

For detailed architecture information, see:
- [`ARCHITECTURE.md`](./ARCHITECTURE.md) - Component structure and patterns
- [`../plans/architecture-implementation.md`](../plans/architecture-implementation.md) - Complete implementation guide

## 🤝 Contributing

1. Follow the established coding standards
2. Write tests for new features
3. Update documentation as needed
4. Use atomic design principles
5. Ensure TypeScript compliance

## 📄 License

This project is part of the DreamFactory software suite. All rights reserved.

---

**Ready for 10-Agent Development Swarm Implementation** 🎯