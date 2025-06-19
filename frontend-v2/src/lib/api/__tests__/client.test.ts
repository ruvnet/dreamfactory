import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { server } from '../../mocks/server';
import { APIClient, initializeAPIClient } from '../client';
import { API_CONFIG } from '../../constants/api';

beforeEach(() => {
  server.listen();
});

afterEach(() => {
  server.resetHandlers();
});

afterEach(() => {
  server.close();
});

describe('APIClient', () => {
  let client: APIClient;

  beforeEach(() => {
    client = new APIClient({
      baseURL: 'http://localhost:3000/api/v2',
      apiKey: 'test-api-key',
      timeout: 5000,
    });
  });

  describe('initialization', () => {
    it('should create client with correct configuration', () => {
      expect(client).toBeInstanceOf(APIClient);
    });

    it('should set session token', () => {
      const token = 'test-token';
      client.setSessionToken(token);
      expect(client.getSessionToken()).toBe(token);
    });

    it('should clear session token', () => {
      client.setSessionToken('test-token');
      client.setSessionToken(null);
      expect(client.getSessionToken()).toBeNull();
    });
  });

  describe('HTTP methods', () => {
    it('should make GET request successfully', async () => {
      const response = await client.get('/system/user');
      
      expect(response).toHaveProperty('resource');
      expect(Array.isArray(response.resource)).toBe(true);
    });

    it('should make POST request successfully', async () => {
      const userData = {
        name: 'Test User',
        first_name: 'Test',
        last_name: 'User',
        email: 'test@example.com',
        is_active: true,
      };

      const response = await client.post('/system/user', userData);
      
      expect(response).toHaveProperty('id');
      expect(response.email).toBe(userData.email);
    });

    it('should make PUT request successfully', async () => {
      const updateData = {
        name: 'Updated User',
        email: 'updated@example.com',
      };

      const response = await client.put('/system/user/1', updateData);
      
      expect(response).toHaveProperty('id', 1);
      expect(response.name).toBe(updateData.name);
    });

    it('should make PATCH request successfully', async () => {
      const patchData = {
        name: 'Patched User',
      };

      const response = await client.patch('/system/user/1', patchData);
      
      expect(response).toHaveProperty('id', 1);
      expect(response.name).toBe(patchData.name);
    });

    it('should make DELETE request successfully', async () => {
      await expect(client.delete('/system/user/1')).resolves.not.toThrow();
    });
  });

  describe('request options', () => {
    it('should handle query parameters', async () => {
      const options = {
        limit: 10,
        offset: 0,
        filter: 'is_active=true',
        sort: 'name',
        fields: 'id,name,email',
        include_count: true,
      };

      const response = await client.get('/system/user', options);
      
      expect(response).toHaveProperty('resource');
      expect(response).toHaveProperty('meta');
    });

    it('should handle additional headers', async () => {
      const options = {
        additionalHeaders: {
          'Custom-Header': 'custom-value',
        },
      };

      await expect(client.get('/system/user', options)).resolves.not.toThrow();
    });

    it('should handle additional parameters', async () => {
      const options = {
        additionalParams: {
          custom_param: 'custom_value',
        },
      };

      await expect(client.get('/system/user', options)).resolves.not.toThrow();
    });
  });

  describe('specialized methods', () => {
    it('should use getAll with default options', async () => {
      const response = await client.getAll('/system/user');
      
      expect(response).toHaveProperty('resource');
      expect(response).toHaveProperty('meta');
      expect(response.meta.count).toBeGreaterThanOrEqual(0);
    });

    it('should use getById', async () => {
      const response = await client.getById('/system/user', 1);
      
      expect(response).toHaveProperty('id', 1);
    });

    it('should handle not found error', async () => {
      await expect(client.getById('/system/user', 999)).rejects.toMatchObject({
        error: {
          status_code: 404,
        },
      });
    });

    it('should use create method', async () => {
      const userData = {
        name: 'New User',
        first_name: 'New',
        last_name: 'User',
        email: 'new@example.com',
        is_active: true,
      };

      const response = await client.create('/system/user', userData);
      
      expect(response).toHaveProperty('id');
      expect(response.email).toBe(userData.email);
    });

    it('should use update method', async () => {
      const updateData = {
        name: 'Updated Name',
      };

      const response = await client.update('/system/user', 1, updateData);
      
      expect(response).toHaveProperty('id', 1);
      expect(response.name).toBe(updateData.name);
    });

    it('should use remove method for single ID', async () => {
      await expect(client.remove('/system/user', 1)).resolves.not.toThrow();
    });

    it('should use remove method for multiple IDs', async () => {
      await expect(client.remove('/system/user', [1, 2])).resolves.not.toThrow();
    });
  });

  describe('file operations', () => {
    it('should upload files', async () => {
      // Create mock file list
      const mockFile = new File(['test content'], 'test.txt', { type: 'text/plain' });
      const mockFileList = Object.assign([mockFile], {
        item: (index: number) => mockFile,
        length: 1,
      }) as FileList;

      await expect(
        client.uploadFile('/files/uploads', mockFileList)
      ).resolves.not.toThrow();
    });

    it('should download files as blob', async () => {
      const blob = await client.downloadFile('/files/test.txt');
      
      expect(blob).toBeInstanceOf(Blob);
    });

    it('should download JSON', async () => {
      const jsonString = await client.downloadJson('/system/user/1');
      
      expect(typeof jsonString).toBe('string');
      expect(() => JSON.parse(jsonString)).not.toThrow();
    });
  });

  describe('error handling', () => {
    it('should handle API error responses', async () => {
      await expect(client.get('/system/user/999')).rejects.toMatchObject({
        error: {
          status_code: 404,
          message: expect.any(String),
        },
      });
    });

    it('should handle network errors', async () => {
      const clientWithInvalidUrl = new APIClient({
        baseURL: 'http://invalid-url:9999',
        apiKey: 'test-key',
        timeout: 100,
      });

      await expect(clientWithInvalidUrl.get('/test')).rejects.toMatchObject({
        error: {
          message: expect.any(String),
        },
      });
    });
  });

  describe('legacy methods', () => {
    it('should use legacy delete method', async () => {
      await expect(
        client.legacyDelete('/system/service', 'test-endpoint')
      ).resolves.not.toThrow();
    });
  });
});

describe('initializeAPIClient', () => {
  it('should initialize and return API client', () => {
    const client = initializeAPIClient({
      baseURL: '/api/v2',
      apiKey: 'test-key',
    });

    expect(client).toBeInstanceOf(APIClient);
  });

  it('should use default timeout if not provided', () => {
    const client = initializeAPIClient({
      baseURL: '/api/v2',
      apiKey: 'test-key',
    });

    expect(client).toBeInstanceOf(APIClient);
  });
});

describe('getAPIClient', () => {
  it('should throw error if not initialized', () => {
    // Reset module to clear previous initialization
    vi.resetModules();
    
    expect(() => {
      const { getAPIClient } = require('../client');
      getAPIClient();
    }).toThrow('API client not initialized');
  });

  it('should return initialized client', () => {
    const { initializeAPIClient, getAPIClient } = require('../client');
    
    initializeAPIClient({
      baseURL: '/api/v2',
      apiKey: 'test-key',
    });

    const client = getAPIClient();
    expect(client).toBeInstanceOf(APIClient);
  });
});