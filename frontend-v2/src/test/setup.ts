import { beforeAll, afterEach, afterAll } from 'vitest';
import { server } from '../lib/mocks/server';

// Establish API mocking before all tests
beforeAll(() => {
  server.listen();
});

// Reset any request handlers that are declared as part of our tests
// (i.e. for testing one-time error scenarios)
afterEach(() => {
  server.resetHandlers();
});

// Clean up after the tests are finished
afterAll(() => {
  server.close();
});

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  observe() {
    return null;
  }
  disconnect() {
    return null;
  }
  unobserve() {
    return null;
  }
};

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  constructor() {}
  observe() {
    return null;
  }
  disconnect() {
    return null;
  }
  unobserve() {
    return null;
  }
};

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock WebSocket
global.WebSocket = class WebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  constructor(url: string) {
    this.url = url;
    this.readyState = WebSocket.CONNECTING;
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      this.onopen?.(new Event('open'));
    }, 0);
  }

  url: string;
  readyState: number;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  send(data: string | ArrayBufferLike | Blob | ArrayBufferView) {
    // Mock send - do nothing
  }

  close(code?: number, reason?: string) {
    this.readyState = WebSocket.CLOSED;
    setTimeout(() => {
      this.onclose?.(new CloseEvent('close', { code, reason }));
    }, 0);
  }

  addEventListener(type: string, listener: EventListener) {
    // Mock addEventListener
  }

  removeEventListener(type: string, listener: EventListener) {
    // Mock removeEventListener
  }

  dispatchEvent(event: Event): boolean {
    return true;
  }
};