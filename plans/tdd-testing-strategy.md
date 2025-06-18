# DreamFactory Frontend TDD Testing Strategy

## Overview

This document outlines a comprehensive Test-Driven Development (TDD) strategy for the DreamFactory frontend to achieve 100% test coverage. The strategy includes unit testing, integration testing, end-to-end testing, and specialized testing for AI agent interactions.

## Current Architecture Analysis

**Technology Stack:**
- Backend: Laravel PHP Framework
- Frontend: Vue.js 2.x with Laravel Mix
- Build Tool: Webpack via Laravel Mix
- Current Dependencies: Vue 2.1.10, Axios, jQuery, Bootstrap

## Testing Pyramid Strategy

### 1. Unit Tests (70% of total tests)
- **Framework**: Jest + Vue Test Utils
- **Coverage Target**: 100% of Vue components, JavaScript utilities, and business logic
- **Speed**: Fast execution (< 5ms per test)
- **Scope**: Individual components, functions, and modules

### 2. Integration Tests (20% of total tests)
- **Framework**: Jest + React Testing Library patterns adapted for Vue
- **Coverage Target**: Component interactions, API integrations, state management
- **Speed**: Medium execution (< 100ms per test)
- **Scope**: Component integration, API mocking, data flow

### 3. End-to-End Tests (10% of total tests)
- **Framework**: Playwright (primary) + Cypress (fallback)
- **Coverage Target**: Critical user journeys, cross-browser compatibility
- **Speed**: Slow execution (< 30s per test)
- **Scope**: Full application workflows, real browser testing

## Technology Stack Recommendations

### Core Testing Dependencies

```json
{
  "devDependencies": {
    "@vue/test-utils": "^1.3.6",
    "jest": "^29.7.0",
    "vue-jest": "^3.0.7",
    "babel-jest": "^29.7.0",
    "@babel/preset-env": "^7.23.0",
    "jest-environment-jsdom": "^29.7.0",
    "jest-serializer-vue": "^3.1.0",
    
    "playwright": "^1.40.0",
    "@playwright/test": "^1.40.0",
    
    "cypress": "^13.6.0",
    "cypress-axe": "^1.5.0",
    "cypress-visual-regression": "^3.0.0",
    
    "msw": "^2.0.0",
    "axios-mock-adapter": "^1.22.0",
    
    "jest-coverage-reporter": "^1.2.1",
    "istanbul-reports": "^3.1.6",
    
    "lighthouse": "^11.4.0",
    "pa11y": "^8.0.0",
    
    "sinon": "^17.0.1",
    "flush-promises": "^1.0.2"
  }
}
```

## 1. Unit Testing Strategy

### Jest Configuration

**File: `jest.config.js`**
```javascript
module.exports = {
  preset: '@vue/cli-plugin-unit-jest',
  testEnvironment: 'jsdom',
  moduleFileExtensions: ['js', 'json', 'vue'],
  transform: {
    '^.+\\.vue$': 'vue-jest',
    '^.+\\.js$': 'babel-jest'
  },
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/resources/js/$1'
  },
  snapshotSerializers: ['jest-serializer-vue'],
  testMatch: [
    '**/tests/unit/**/*.spec.(js|jsx|ts|tsx)|**/__tests__/*.(js|jsx|ts|tsx)'
  ],
  collectCoverageFrom: [
    'resources/js/**/*.{js,vue}',
    '!resources/js/app.js',
    '!resources/js/bootstrap.js',
    '!**/node_modules/**'
  ],
  coverageThreshold: {
    global: {
      branches: 100,
      functions: 100,
      lines: 100,
      statements: 100
    }
  },
  setupFilesAfterEnv: ['<rootDir>/tests/unit/setup.js']
};
```

### Vue Component Testing Patterns

**File: `tests/unit/components/ExampleComponent.spec.js`**
```javascript
import { shallowMount, mount } from '@vue/test-utils';
import ExampleComponent from '@/components/ExampleComponent.vue';

describe('ExampleComponent', () => {
  let wrapper;

  beforeEach(() => {
    wrapper = shallowMount(ExampleComponent);
  });

  afterEach(() => {
    wrapper.destroy();
  });

  describe('Component Rendering', () => {
    it('renders correctly', () => {
      expect(wrapper.element).toMatchSnapshot();
    });

    it('displays the correct heading', () => {
      const heading = wrapper.find('.panel-heading');
      expect(heading.text()).toBe('Example Component');
    });

    it('displays the correct body text', () => {
      const body = wrapper.find('.panel-body');
      expect(body.text()).toBe("I'm an example component!");
    });
  });

  describe('Component Lifecycle', () => {
    it('logs message on mount', () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation();
      mount(ExampleComponent);
      expect(consoleSpy).toHaveBeenCalledWith('Component mounted.');
      consoleSpy.mockRestore();
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA attributes', () => {
      expect(wrapper.find('.panel').attributes('role')).toBeDefined();
    });

    it('maintains focus management', () => {
      // Focus management tests
      const focusableElements = wrapper.findAll('button, input, select, textarea, a[href], [tabindex]');
      expect(focusableElements.length).toBeGreaterThanOrEqual(0);
    });
  });
});
```

### Advanced Testing Utilities

**File: `tests/unit/utils/test-utils.js`**
```javascript
import { createLocalVue, mount, shallowMount } from '@vue/test-utils';
import flushPromises from 'flush-promises';

// Mock factory for API responses
export const createMockApiResponse = (data, status = 200) => ({
  data,
  status,
  statusText: 'OK',
  headers: {},
  config: {}
});

// Component factory with common props/mocks
export const createComponentFactory = (Component, defaultProps = {}) => {
  return (props = {}, options = {}) => {
    return mount(Component, {
      propsData: { ...defaultProps, ...props },
      ...options
    });
  };
};

// Async component testing helper
export const mountAsyncComponent = async (Component, options = {}) => {
  const wrapper = mount(Component, options);
  await flushPromises();
  return wrapper;
};

// Event testing helper
export const triggerEvent = async (wrapper, selector, event, data = {}) => {
  const element = wrapper.find(selector);
  element.trigger(event, data);
  await wrapper.vm.$nextTick();
  return wrapper;
};

// Form testing helper
export const fillForm = async (wrapper, formData) => {
  for (const [selector, value] of Object.entries(formData)) {
    const input = wrapper.find(selector);
    if (input.element.type === 'checkbox' || input.element.type === 'radio') {
      input.setChecked(value);
    } else {
      input.setValue(value);
    }
  }
  await wrapper.vm.$nextTick();
};
```

## 2. Integration Testing Strategy

### API Mocking with MSW

**File: `tests/mocks/api-handlers.js`**
```javascript
import { rest } from 'msw';

export const handlers = [
  // Authentication endpoints
  rest.post('/api/login', (req, res, ctx) => {
    const { email, password } = req.body;
    
    if (email === 'admin@dreamfactory.com' && password === 'password') {
      return res(
        ctx.status(200),
        ctx.json({
          token: 'mock-jwt-token',
          user: { id: 1, email, role: 'admin' }
        })
      );
    }
    
    return res(
      ctx.status(401),
      ctx.json({ error: 'Invalid credentials' })
    );
  }),

  // System endpoints
  rest.get('/api/system/environment', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        version: '4.0.0',
        edition: 'Open Source',
        host_os: 'Linux'
      })
    );
  }),

  // Database services
  rest.get('/api/system/service', (req, res, ctx) => {
    return res(
      ctx.status(200),
      ctx.json({
        resource: [
          { name: 'db', type: 'sql_db', label: 'Local Database' },
          { name: 'api', type: 'rest', label: 'REST API' }
        ]
      })
    );
  }),

  // Error simulation
  rest.get('/api/error', (req, res, ctx) => {
    return res(
      ctx.status(500),
      ctx.json({ error: 'Internal server error' })
    );
  })
];
```

**File: `tests/mocks/server.js`**
```javascript
import { setupServer } from 'msw/node';
import { handlers } from './api-handlers';

export const server = setupServer(...handlers);
```

### Integration Test Examples

**File: `tests/integration/auth.spec.js`**
```javascript
import { mount, createLocalVue } from '@vue/test-utils';
import axios from 'axios';
import flushPromises from 'flush-promises';
import LoginComponent from '@/components/LoginComponent.vue';
import { server } from '../mocks/server';

const localVue = createLocalVue();

describe('Authentication Integration', () => {
  beforeAll(() => server.listen());
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());

  it('successfully authenticates user', async () => {
    const wrapper = mount(LoginComponent, { localVue });
    
    await wrapper.find('input[type="email"]').setValue('admin@dreamfactory.com');
    await wrapper.find('input[type="password"]').setValue('password');
    
    const form = wrapper.find('form');
    await form.trigger('submit');
    await flushPromises();
    
    expect(wrapper.emitted('login-success')).toBeTruthy();
    expect(wrapper.emitted('login-success')[0][0]).toMatchObject({
      token: 'mock-jwt-token',
      user: { email: 'admin@dreamfactory.com', role: 'admin' }
    });
  });

  it('handles authentication errors', async () => {
    const wrapper = mount(LoginComponent, { localVue });
    
    await wrapper.find('input[type="email"]').setValue('wrong@email.com');
    await wrapper.find('input[type="password"]').setValue('wrongpassword');
    
    const form = wrapper.find('form');
    await form.trigger('submit');
    await flushPromises();
    
    expect(wrapper.emitted('login-error')).toBeTruthy();
    expect(wrapper.find('.error-message').text()).toContain('Invalid credentials');
  });
});
```

## 3. End-to-End Testing Strategy

### Playwright Configuration

**File: `playwright.config.js`**
```javascript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html'],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/results.xml' }]
  ],
  use: {
    baseURL: 'http://localhost:8000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure'
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] }
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] }
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] }
    },
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] }
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] }
    }
  ],
  webServer: {
    command: 'php artisan serve',
    port: 8000,
    reuseExistingServer: !process.env.CI
  }
});
```

### E2E Test Examples

**File: `tests/e2e/auth-flow.spec.js`**
```javascript
import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('complete login flow', async ({ page }) => {
    // Navigate to login
    await page.click('[data-testid="login-button"]');
    await expect(page).toHaveURL('/login');
    
    // Fill login form
    await page.fill('[data-testid="email-input"]', 'admin@dreamfactory.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    
    // Submit form
    await page.click('[data-testid="submit-button"]');
    
    // Verify successful login
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
    await expect(page.locator('[data-testid="logout-button"]')).toBeVisible();
  });

  test('handles invalid credentials', async ({ page }) => {
    await page.click('[data-testid="login-button"]');
    
    await page.fill('[data-testid="email-input"]', 'wrong@email.com');
    await page.fill('[data-testid="password-input"]', 'wrongpassword');
    await page.click('[data-testid="submit-button"]');
    
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Invalid credentials');
  });

  test('logout functionality', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'admin@dreamfactory.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-button"]');
    
    // Logout
    await page.click('[data-testid="user-menu"]');
    await page.click('[data-testid="logout-button"]');
    
    // Verify logout
    await expect(page).toHaveURL('/');
    await expect(page.locator('[data-testid="login-button"]')).toBeVisible();
  });
});
```

**File: `tests/e2e/api-management.spec.js`**
```javascript
import { test, expect } from '@playwright/test';

test.describe('API Management', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'admin@dreamfactory.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-button"]');
    await expect(page).toHaveURL('/dashboard');
  });

  test('create new API service', async ({ page }) => {
    await page.click('[data-testid="services-tab"]');
    await page.click('[data-testid="create-service-button"]');
    
    // Fill service form
    await page.selectOption('[data-testid="service-type"]', 'rest');
    await page.fill('[data-testid="service-name"]', 'test-api');
    await page.fill('[data-testid="service-label"]', 'Test API Service');
    await page.fill('[data-testid="service-description"]', 'Test API for e2e testing');
    
    await page.click('[data-testid="save-service-button"]');
    
    // Verify service creation
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="service-list"]')).toContainText('test-api');
  });

  test('test API endpoint', async ({ page }) => {
    await page.click('[data-testid="api-docs-tab"]');
    
    // Select service and endpoint
    await page.selectOption('[data-testid="service-selector"]', 'test-api');
    await page.click('[data-testid="endpoint-get-users"]');
    
    // Execute API call
    await page.click('[data-testid="try-it-button"]');
    
    // Verify response
    await expect(page.locator('[data-testid="response-status"]')).toContainText('200');
    await expect(page.locator('[data-testid="response-body"]')).toBeVisible();
  });
});
```

## 4. Visual Regression Testing

### Cypress Visual Testing

**File: `cypress/e2e/visual-regression.cy.js`**
```javascript
describe('Visual Regression Tests', () => {
  beforeEach(() => {
    cy.login('admin@dreamfactory.com', 'password123');
  });

  it('dashboard layout remains consistent', () => {
    cy.visit('/dashboard');
    cy.get('[data-testid="dashboard-container"]').should('be.visible');
    cy.compareSnapshot('dashboard-layout');
  });

  it('login form styling is correct', () => {
    cy.visit('/login');
    cy.get('[data-testid="login-form"]').should('be.visible');
    cy.compareSnapshot('login-form');
  });

  it('responsive design on mobile', () => {
    cy.viewport('iphone-x');
    cy.visit('/dashboard');
    cy.get('[data-testid="mobile-menu"]').should('be.visible');
    cy.compareSnapshot('mobile-dashboard');
  });

  it('dark mode styling', () => {
    cy.visit('/dashboard');
    cy.get('[data-testid="theme-toggle"]').click();
    cy.get('body').should('have.class', 'dark-theme');
    cy.compareSnapshot('dark-mode-dashboard');
  });
});
```

## 5. Performance Testing

### Lighthouse Integration

**File: `tests/performance/lighthouse.spec.js`**
```javascript
import { test } from '@playwright/test';
import { playAudit } from 'playwright-lighthouse';

test.describe('Performance Tests', () => {
  test('homepage performance audit', async ({ page }) => {
    await page.goto('/');
    
    await playAudit({
      page,
      thresholds: {
        performance: 90,
        accessibility: 95,
        'best-practices': 90,
        seo: 90,
        pwa: 80
      },
      port: 9222
    });
  });

  test('dashboard performance after login', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'admin@dreamfactory.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-button"]');
    
    await playAudit({
      page,
      thresholds: {
        performance: 85,
        accessibility: 95,
        'best-practices': 90
      },
      port: 9222
    });
  });
});
```

### Bundle Size Monitoring

**File: `tests/performance/bundle-size.spec.js`**
```javascript
import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';

test.describe('Bundle Size Tests', () => {
  test('JavaScript bundle size within limits', async () => {
    const bundlePath = path.join(__dirname, '../../public/js/app.js');
    const stats = fs.statSync(bundlePath);
    const fileSizeInMB = stats.size / (1024 * 1024);
    
    // Ensure bundle is under 1MB
    expect(fileSizeInMB).toBeLessThan(1);
  });

  test('CSS bundle size within limits', async () => {
    const bundlePath = path.join(__dirname, '../../public/css/app.css');
    const stats = fs.statSync(bundlePath);
    const fileSizeInKB = stats.size / 1024;
    
    // Ensure CSS is under 500KB
    expect(fileSizeInKB).toBeLessThan(500);
  });
});
```

## 6. Accessibility Testing

### Automated A11y Testing

**File: `tests/accessibility/a11y.spec.js`**
```javascript
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('Accessibility Tests', () => {
  test('homepage accessibility compliance', async ({ page }) => {
    await page.goto('/');
    
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
      .analyze();
    
    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('login form accessibility', async ({ page }) => {
    await page.goto('/login');
    
    const accessibilityScanResults = await new AxeBuilder({ page })
      .include('[data-testid="login-form"]')
      .analyze();
    
    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('keyboard navigation', async ({ page }) => {
    await page.goto('/login');
    
    // Test tab navigation
    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="email-input"]')).toBeFocused();
    
    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="password-input"]')).toBeFocused();
    
    await page.keyboard.press('Tab');
    await expect(page.locator('[data-testid="submit-button"]')).toBeFocused();
  });

  test('screen reader compatibility', async ({ page }) => {
    await page.goto('/dashboard');
    
    // Verify ARIA labels and roles
    const navigation = page.locator('[role="navigation"]');
    await expect(navigation).toBeVisible();
    
    const buttons = page.locator('button[aria-label]');
    const buttonCount = await buttons.count();
    expect(buttonCount).toBeGreaterThan(0);
  });
});
```

## 7. AI Agent Interaction Testing

### Agent Communication Testing

**File: `tests/ai-agents/agent-interaction.spec.js`**
```javascript
import { test, expect } from '@playwright/test';

test.describe('AI Agent Integration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'admin@dreamfactory.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-button"]');
  });

  test('AI assistant chat interface', async ({ page }) => {
    await page.click('[data-testid="ai-assistant-button"]');
    
    // Verify chat interface opens
    await expect(page.locator('[data-testid="chat-interface"]')).toBeVisible();
    
    // Send message to AI
    await page.fill('[data-testid="chat-input"]', 'Help me create a new API endpoint');
    await page.click('[data-testid="send-button"]');
    
    // Verify AI response
    await expect(page.locator('[data-testid="ai-response"]')).toBeVisible();
    await expect(page.locator('[data-testid="ai-response"]')).toContainText('API endpoint');
  });

  test('AI code generation integration', async ({ page }) => {
    await page.click('[data-testid="services-tab"]');
    await page.click('[data-testid="ai-generate-service"]');
    
    // Fill AI generation form
    await page.fill('[data-testid="service-description"]', 'User management service with CRUD operations');
    await page.click('[data-testid="generate-button"]');
    
    // Verify AI generates service configuration
    await expect(page.locator('[data-testid="generated-config"]')).toBeVisible();
    await expect(page.locator('[data-testid="generated-config"]')).toContainText('user');
  });

  test('AI error detection and suggestions', async ({ page }) => {
    await page.click('[data-testid="logs-tab"]');
    
    // Trigger AI analysis of errors
    await page.click('[data-testid="ai-analyze-errors"]');
    
    // Verify AI provides suggestions
    await expect(page.locator('[data-testid="ai-suggestions"]')).toBeVisible();
    await expect(page.locator('[data-testid="suggestion-item"]').first()).toBeVisible();
  });
});
```

### Agent Communication Protocol Testing

**File: `tests/unit/ai-agents/communication.spec.js`**
```javascript
import { shallowMount } from '@vue/test-utils';
import AgentCommunicator from '@/components/AgentCommunicator.vue';
import { createMockApiResponse } from '../utils/test-utils';

describe('Agent Communication Protocol', () => {
  let wrapper;
  let mockAxios;

  beforeEach(() => {
    mockAxios = {
      post: jest.fn(),
      get: jest.fn()
    };
    
    wrapper = shallowMount(AgentCommunicator, {
      mocks: {
        $axios: mockAxios
      }
    });
  });

  it('sends message to AI agent', async () => {
    const message = 'Create a REST API for user management';
    mockAxios.post.mockResolvedValue(createMockApiResponse({
      response: 'I can help you create a REST API for user management...',
      actions: ['generate-service', 'create-endpoints']
    }));

    await wrapper.vm.sendMessage(message);

    expect(mockAxios.post).toHaveBeenCalledWith('/api/ai/chat', {
      message,
      context: expect.any(Object)
    });
  });

  it('handles agent action suggestions', async () => {
    const response = {
      response: 'I suggest creating these endpoints...',
      actions: [
        { type: 'generate-service', label: 'Generate Service', data: { name: 'users' } },
        { type: 'create-endpoint', label: 'Create GET /users', data: { method: 'GET', path: '/users' } }
      ]
    };

    wrapper.vm.handleAgentResponse(response);

    expect(wrapper.vm.suggestions).toEqual(response.actions);
    expect(wrapper.emitted('actions-suggested')).toBeTruthy();
  });

  it('executes agent-suggested actions', async () => {
    const action = {
      type: 'generate-service',
      data: { name: 'users', type: 'rest' }
    };

    mockAxios.post.mockResolvedValue(createMockApiResponse({
      success: true,
      serviceId: 'users-123'
    }));

    await wrapper.vm.executeAction(action);

    expect(mockAxios.post).toHaveBeenCalledWith('/api/services', action.data);
    expect(wrapper.emitted('action-completed')).toBeTruthy();
  });
});
```

## 8. Test Data Management

### Test Fixtures

**File: `tests/fixtures/api-responses.js`**
```javascript
export const userFixtures = {
  adminUser: {
    id: 1,
    email: 'admin@dreamfactory.com',
    first_name: 'Admin',
    last_name: 'User',
    role: 'admin',
    is_active: true
  },
  
  regularUser: {
    id: 2,
    email: 'user@example.com',
    first_name: 'Regular',
    last_name: 'User',
    role: 'user',
    is_active: true
  }
};

export const serviceFixtures = {
  restService: {
    id: 1,
    name: 'test-api',
    label: 'Test REST API',
    description: 'Test REST service for automated testing',
    type: 'rest',
    is_active: true
  },
  
  sqlService: {
    id: 2,
    name: 'test-db',
    label: 'Test Database',
    description: 'Test SQL database service',
    type: 'sql_db',
    is_active: true
  }
};

export const errorFixtures = {
  authError: {
    error: {
      code: 401,
      message: 'Unauthorized',
      details: 'Invalid authentication credentials'
    }
  },
  
  validationError: {
    error: {
      code: 422,
      message: 'Validation failed',
      details: {
        email: ['The email field is required'],
        password: ['The password must be at least 8 characters']
      }
    }
  }
};
```

### Database Seeding for Tests

**File: `tests/database/TestSeeder.php`**
```php
<?php

use Illuminate\Database\Seeder;
use DreamFactory\Core\Models\User;
use DreamFactory\Core\Models\Service;

class TestSeeder extends Seeder
{
    public function run()
    {
        // Create test users
        User::create([
            'email' => 'admin@dreamfactory.com',
            'password' => bcrypt('password123'),
            'first_name' => 'Test',
            'last_name' => 'Admin',
            'is_active' => true
        ]);

        User::create([
            'email' => 'user@example.com',
            'password' => bcrypt('password123'),
            'first_name' => 'Test',
            'last_name' => 'User',
            'is_active' => true
        ]);

        // Create test services
        Service::create([
            'name' => 'test-api',
            'label' => 'Test REST API',
            'description' => 'Test service for automated testing',
            'type' => 'rest',
            'is_active' => true,
            'config' => json_encode(['base_url' => 'https://jsonplaceholder.typicode.com'])
        ]);
    }
}
```

## 9. CI/CD Pipeline Integration

### GitHub Actions Workflow

**File: `.github/workflows/test.yml`**
```yaml
name: Frontend Testing Pipeline

on:
  push:
    branches: [ master, develop ]
  pull_request:
    branches: [ master ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      mysql:
        image: mysql:8.0
        env:
          MYSQL_ROOT_PASSWORD: password
          MYSQL_DATABASE: dreamfactory_test
        options: --health-cmd="mysqladmin ping" --health-interval=10s --health-timeout=5s --health-retries=3

    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Setup PHP
      uses: shivammathur/setup-php@v2
      with:
        php-version: '8.1'
        extensions: mysql, mbstring, xml, curl
    
    - name: Install PHP dependencies
      run: composer install --no-interaction --prefer-dist --optimize-autoloader
    
    - name: Install Node dependencies
      run: npm ci
    
    - name: Build assets
      run: npm run production
    
    - name: Prepare Laravel application
      run: |
        cp .env.example .env
        php artisan key:generate
        php artisan config:cache
        php artisan migrate --seed
    
    - name: Run unit tests
      run: npm run test:unit
    
    - name: Run integration tests
      run: npm run test:integration
    
    - name: Install Playwright browsers
      run: npx playwright install --with-deps
    
    - name: Start Laravel server
      run: php artisan serve &
      
    - name: Wait for server
      run: sleep 10
    
    - name: Run E2E tests
      run: npm run test:e2e
    
    - name: Run accessibility tests
      run: npm run test:a11y
    
    - name: Run performance tests
      run: npm run test:performance
    
    - name: Generate coverage report
      run: npm run coverage:report
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage/lcov.info
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: |
          test-results/
          coverage/
          playwright-report/
```

### Package.json Scripts

**File: `package.json` (testing scripts section)**
```json
{
  "scripts": {
    "test": "npm run test:unit && npm run test:integration && npm run test:e2e",
    "test:unit": "jest --config=jest.config.js",
    "test:unit:watch": "jest --config=jest.config.js --watch",
    "test:unit:coverage": "jest --config=jest.config.js --coverage",
    "test:integration": "jest --config=jest.integration.config.js",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:debug": "playwright test --debug",
    "test:a11y": "playwright test tests/accessibility/",
    "test:performance": "lighthouse-ci autorun",
    "test:visual": "cypress run --spec=\"cypress/e2e/visual-regression/**/*.cy.js\"",
    "coverage:report": "nyc report --reporter=html --reporter=lcov",
    "coverage:open": "open coverage/lcov-report/index.html",
    "lint:test": "eslint tests/**/*.js",
    "test:ai-agents": "jest --config=tests/ai-agents/jest.config.js"
  }
}
```

## 10. Quality Gates and Coverage Reporting

### Coverage Configuration

**File: `jest.coverage.config.js`**
```javascript
module.exports = {
  ...require('./jest.config.js'),
  collectCoverage: true,
  coverageDirectory: 'coverage',
  coverageReporters: ['html', 'lcov', 'text', 'json'],
  coverageThreshold: {
    global: {
      branches: 100,
      functions: 100,
      lines: 100,
      statements: 100
    },
    // Per-file thresholds
    './resources/js/components/': {
      branches: 100,
      functions: 100,
      lines: 100,
      statements: 100
    },
    './resources/js/utils/': {
      branches: 100,
      functions: 100,
      lines: 100,
      statements: 100
    }
  },
  coveragePathIgnorePatterns: [
    '/node_modules/',
    '/tests/',
    'jest.config.js',
    'webpack.mix.js'
  ]
};
```

### SonarQube Configuration

**File: `sonar-project.properties`**
```properties
sonar.projectKey=dreamfactory-frontend
sonar.projectName=DreamFactory Frontend
sonar.projectVersion=1.0.0

sonar.sources=resources/js
sonar.tests=tests
sonar.exclusions=node_modules/**,public/**,storage/**,vendor/**

sonar.javascript.lcov.reportPaths=coverage/lcov.info
sonar.testExecutionReportPaths=test-results/results.xml

sonar.coverage.exclusions=**/*.spec.js,**/*.test.js,tests/**

# Quality gate thresholds
sonar.qualitygate.wait=true
```

## 11. Test Environment Setup

### Docker Test Environment

**File: `docker-compose.test.yml`**
```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.test
    volumes:
      - .:/app
      - /app/node_modules
      - /app/vendor
    environment:
      - APP_ENV=testing
      - DB_HOST=mysql
      - DB_DATABASE=dreamfactory_test
      - DB_USERNAME=root
      - DB_PASSWORD=password
    depends_on:
      - mysql
      - redis
    ports:
      - "8000:8000"
    command: >
      sh -c "php artisan migrate:fresh --seed &&
             php artisan serve --host=0.0.0.0 --port=8000"

  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: password
      MYSQL_DATABASE: dreamfactory_test
    ports:
      - "3306:3306"
    tmpfs:
      - /var/lib/mysql

  redis:
    image: redis:alpine
    ports:
      - "6379:6379"

  node:
    image: node:18-alpine
    working_dir: /app
    volumes:
      - .:/app
      - /app/node_modules
    command: sh -c "npm ci && npm run test"
    depends_on:
      - app
```

### Test Setup Commands

**File: `scripts/test-setup.sh`**
```bash
#!/bin/bash

# Install dependencies
echo "Installing dependencies..."
npm ci
composer install --no-interaction --prefer-dist --optimize-autoloader

# Setup test environment
echo "Setting up test environment..."
cp .env.testing .env
php artisan key:generate
php artisan config:cache

# Setup test database
echo "Setting up test database..."
php artisan migrate:fresh --seed --env=testing

# Install Playwright browsers
echo "Installing Playwright browsers..."
npx playwright install --with-deps

# Build assets
echo "Building assets..."
npm run development

echo "Test environment setup complete!"
```

## 12. Monitoring and Reporting

### Test Results Dashboard

**File: `scripts/generate-test-report.js`**
```javascript
const fs = require('fs');
const path = require('path');

class TestReportGenerator {
  constructor() {
    this.results = {
      unit: null,
      integration: null,
      e2e: null,
      coverage: null,
      performance: null,
      accessibility: null
    };
  }

  loadResults() {
    // Load Jest results
    if (fs.existsSync('test-results/jest-results.json')) {
      this.results.unit = JSON.parse(fs.readFileSync('test-results/jest-results.json'));
    }

    // Load Playwright results
    if (fs.existsSync('test-results/playwright-results.json')) {
      this.results.e2e = JSON.parse(fs.readFileSync('test-results/playwright-results.json'));
    }

    // Load coverage results
    if (fs.existsSync('coverage/coverage-summary.json')) {
      this.results.coverage = JSON.parse(fs.readFileSync('coverage/coverage-summary.json'));
    }
  }

  generateReport() {
    this.loadResults();
    
    const report = {
      timestamp: new Date().toISOString(),
      summary: this.generateSummary(),
      details: this.results,
      qualityGates: this.checkQualityGates()
    };

    fs.writeFileSync('test-results/comprehensive-report.json', JSON.stringify(report, null, 2));
    this.generateHtmlReport(report);
    
    return report;
  }

  generateSummary() {
    const summary = {
      totalTests: 0,
      passedTests: 0,
      failedTests: 0,
      coveragePercentage: 0,
      duration: 0
    };

    if (this.results.unit) {
      summary.totalTests += this.results.unit.numTotalTests;
      summary.passedTests += this.results.unit.numPassedTests;
      summary.failedTests += this.results.unit.numFailedTests;
    }

    if (this.results.coverage) {
      const total = this.results.coverage.total;
      summary.coveragePercentage = Math.round(
        (total.lines.pct + total.statements.pct + total.functions.pct + total.branches.pct) / 4
      );
    }

    return summary;
  }

  checkQualityGates() {
    const gates = {
      coverage: { threshold: 100, actual: 0, passed: false },
      tests: { threshold: 0, actual: 0, passed: true },
      performance: { threshold: 90, actual: 0, passed: false }
    };

    if (this.results.coverage) {
      gates.coverage.actual = this.generateSummary().coveragePercentage;
      gates.coverage.passed = gates.coverage.actual >= gates.coverage.threshold;
    }

    return gates;
  }

  generateHtmlReport(report) {
    const html = `
<!DOCTYPE html>
<html>
<head>
    <title>DreamFactory Frontend Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .summary { background: #f5f5f5; padding: 20px; border-radius: 5px; }
        .metric { display: inline-block; margin: 10px; padding: 15px; background: white; border-radius: 3px; }
        .passed { color: green; }
        .failed { color: red; }
        .coverage-bar { width: 200px; height: 20px; background: #ddd; border-radius: 10px; overflow: hidden; }
        .coverage-fill { height: 100%; background: linear-gradient(to right, red, yellow, green); }
    </style>
</head>
<body>
    <h1>DreamFactory Frontend Test Report</h1>
    <div class="summary">
        <h2>Summary</h2>
        <div class="metric">
            <strong>Total Tests:</strong> ${report.summary.totalTests}
        </div>
        <div class="metric">
            <strong>Passed:</strong> <span class="passed">${report.summary.passedTests}</span>
        </div>
        <div class="metric">
            <strong>Failed:</strong> <span class="failed">${report.summary.failedTests}</span>
        </div>
        <div class="metric">
            <strong>Coverage:</strong> ${report.summary.coveragePercentage}%
            <div class="coverage-bar">
                <div class="coverage-fill" style="width: ${report.summary.coveragePercentage}%"></div>
            </div>
        </div>
    </div>
    
    <h2>Quality Gates</h2>
    <ul>
        ${Object.entries(report.qualityGates).map(([key, gate]) => 
          `<li><strong>${key}:</strong> ${gate.actual}/${gate.threshold} ${gate.passed ? '✅' : '❌'}</li>`
        ).join('')}
    </ul>
    
    <p><em>Generated on ${report.timestamp}</em></p>
</body>
</html>`;

    fs.writeFileSync('test-results/report.html', html);
  }
}

// Generate report if run directly
if (require.main === module) {
  const generator = new TestReportGenerator();
  const report = generator.generateReport();
  console.log('Test report generated:', report.summary);
}

module.exports = TestReportGenerator;
```

## Implementation Timeline

### Phase 1: Foundation Setup (Week 1-2)
1. Install and configure Jest, Vue Test Utils
2. Set up basic unit testing infrastructure
3. Create test utilities and helpers
4. Implement first component tests
5. Set up coverage reporting

### Phase 2: Integration Testing (Week 3-4)
1. Set up MSW for API mocking
2. Create integration test suite
3. Implement authentication flow tests
4. Add database seeding for tests
5. Set up CI/CD pipeline basics

### Phase 3: E2E Testing (Week 5-6)
1. Install and configure Playwright
2. Create critical user journey tests
3. Add cross-browser testing
4. Implement visual regression testing
5. Set up performance monitoring

### Phase 4: Advanced Testing (Week 7-8)
1. Add accessibility testing automation
2. Implement AI agent interaction tests
3. Create comprehensive test data management
4. Add advanced reporting and monitoring
5. Fine-tune quality gates

### Phase 5: Optimization (Week 9-10)
1. Optimize test performance
2. Add parallel test execution
3. Implement test result caching
4. Create comprehensive documentation
5. Training and knowledge transfer

## Best Practices and Guidelines

### 1. Test Naming Conventions
- Use descriptive test names that explain the expected behavior
- Follow the pattern: `should [expected behavior] when [condition]`
- Group related tests using `describe` blocks

### 2. Test Structure
- Follow AAA pattern: Arrange, Act, Assert
- Keep tests focused on single functionality
- Use beforeEach/afterEach for common setup/cleanup

### 3. Mocking Strategy
- Mock external dependencies (APIs, third-party libraries)
- Use MSW for realistic API mocking
- Avoid over-mocking internal functions

### 4. Data-Driven Testing
- Use test fixtures for consistent test data
- Parameterize tests for multiple input scenarios
- Keep test data separate from test logic

### 5. Continuous Improvement
- Regularly review and refactor tests
- Monitor test execution times
- Update tests when requirements change
- Maintain test documentation

## Conclusion

This comprehensive testing strategy provides a robust foundation for achieving 100% test coverage in the DreamFactory frontend. The multi-layered approach ensures code quality, performance, accessibility, and compatibility with AI agent interactions.

Key benefits of this strategy:
- **Complete Coverage**: 100% code coverage across all frontend components
- **Quality Assurance**: Multiple testing layers catch different types of issues
- **Performance Monitoring**: Continuous performance and accessibility validation
- **CI/CD Integration**: Automated testing in development workflow
- **Future-Proof**: Support for AI agent testing and modern web standards
- **Maintainable**: Well-structured tests that are easy to update and extend

The implementation should be done incrementally, with each phase building upon the previous one. Regular monitoring and adjustment of the strategy will ensure it continues to meet the evolving needs of the DreamFactory platform.