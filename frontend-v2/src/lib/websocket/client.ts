import { getAPIClient } from '../api/client';
import { invalidateQueries } from '../api/queryClient';

// WebSocket event types
export interface WebSocketEvent {
  type: string;
  data: any;
  timestamp: number;
  service?: string;
  resource?: string;
  user_id?: number;
}

// WebSocket connection states
export enum WebSocketState {
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  ERROR = 'error',
  RECONNECTING = 'reconnecting',
}

// Event listener type
export type WebSocketEventListener = (event: WebSocketEvent) => void;

// WebSocket client configuration
interface WebSocketClientConfig {
  url?: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
  debug?: boolean;
}

// WebSocket client class
export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectInterval: number;
  private maxReconnectAttempts: number;
  private heartbeatInterval: number;
  private debug: boolean;
  
  private state: WebSocketState = WebSocketState.DISCONNECTED;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  
  private eventListeners: Map<string, Set<WebSocketEventListener>> = new Map();
  private stateListeners: Set<(state: WebSocketState) => void> = new Set();
  
  constructor(config: WebSocketClientConfig = {}) {
    this.url = config.url || this.getWebSocketUrl();
    this.reconnectInterval = config.reconnectInterval || 5000;
    this.maxReconnectAttempts = config.maxReconnectAttempts || 10;
    this.heartbeatInterval = config.heartbeatInterval || 30000;
    this.debug = config.debug || false;
  }
  
  private getWebSocketUrl(): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    return `${protocol}//${host}/ws`;
  }
  
  private log(message: string, ...args: any[]) {
    if (this.debug) {
      console.log(`[WebSocket] ${message}`, ...args);
    }
  }
  
  private setState(newState: WebSocketState) {
    if (this.state !== newState) {
      this.state = newState;
      this.log(`State changed to: ${newState}`);
      this.stateListeners.forEach(listener => listener(newState));
    }
  }
  
  // Connect to WebSocket
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        resolve();
        return;
      }
      
      this.setState(WebSocketState.CONNECTING);
      this.log('Connecting to:', this.url);
      
      try {
        this.ws = new WebSocket(this.url);
        
        this.ws.onopen = () => {
          this.log('Connected successfully');
          this.setState(WebSocketState.CONNECTED);
          this.reconnectAttempts = 0;
          this.startHeartbeat();
          this.authenticateConnection();
          resolve();
        };
        
        this.ws.onmessage = (event) => {
          this.handleMessage(event);
        };
        
        this.ws.onclose = (event) => {
          this.log('Connection closed:', event.code, event.reason);
          this.setState(WebSocketState.DISCONNECTED);
          this.stopHeartbeat();
          this.scheduleReconnect();
        };
        
        this.ws.onerror = (error) => {
          this.log('Connection error:', error);
          this.setState(WebSocketState.ERROR);
          reject(error);
        };
        
      } catch (error) {
        this.log('Failed to create WebSocket:', error);
        this.setState(WebSocketState.ERROR);
        reject(error);
      }
    });
  }
  
  // Disconnect from WebSocket
  disconnect() {
    this.log('Disconnecting...');
    
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    
    this.stopHeartbeat();
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    
    this.setState(WebSocketState.DISCONNECTED);
  }
  
  // Send message to server
  send(message: any) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const payload = JSON.stringify(message);
      this.log('Sending message:', payload);
      this.ws.send(payload);
    } else {
      this.log('Cannot send message: WebSocket not connected');
    }
  }
  
  // Authenticate the WebSocket connection
  private authenticateConnection() {
    const apiClient = getAPIClient();
    const sessionToken = apiClient.getSessionToken();
    
    if (sessionToken) {
      this.send({
        type: 'auth',
        token: sessionToken,
      });
    }
  }
  
  // Handle incoming messages
  private handleMessage(event: MessageEvent) {
    try {
      const data = JSON.parse(event.data);
      this.log('Received message:', data);
      
      const wsEvent: WebSocketEvent = {
        type: data.type || 'unknown',
        data: data.data || data,
        timestamp: Date.now(),
        service: data.service,
        resource: data.resource,
        user_id: data.user_id,
      };
      
      // Handle system events
      this.handleSystemEvent(wsEvent);
      
      // Notify listeners
      const listeners = this.eventListeners.get(wsEvent.type) || new Set();
      const allListeners = this.eventListeners.get('*') || new Set();
      
      [...listeners, ...allListeners].forEach(listener => {
        try {
          listener(wsEvent);
        } catch (error) {
          this.log('Error in event listener:', error);
        }
      });
      
    } catch (error) {
      this.log('Error parsing message:', error);
    }
  }
  
  // Handle system events (data invalidation, etc.)
  private handleSystemEvent(event: WebSocketEvent) {
    switch (event.type) {
      case 'user.created':
      case 'user.updated':
      case 'user.deleted':
        invalidateQueries.users();
        if (event.data?.id) {
          invalidateQueries.user(event.data.id);
        }
        break;
        
      case 'service.created':
      case 'service.updated':
      case 'service.deleted':
        invalidateQueries.services();
        if (event.data?.id) {
          invalidateQueries.service(event.data.id);
        }
        break;
        
      case 'role.created':
      case 'role.updated':
      case 'role.deleted':
        invalidateQueries.roles();
        if (event.data?.id) {
          invalidateQueries.role(event.data.id);
        }
        break;
        
      case 'app.created':
      case 'app.updated':
      case 'app.deleted':
        invalidateQueries.apps();
        if (event.data?.id) {
          invalidateQueries.app(event.data.id);
        }
        break;
        
      case 'config.updated':
        invalidateQueries.config();
        break;
        
      case 'files.changed':
        if (event.data?.path) {
          invalidateQueries.files(event.data.path);
        } else {
          invalidateQueries.files();
        }
        break;
        
      case 'system.updated':
        invalidateQueries.system();
        break;
        
      case 'heartbeat':
        // Respond to server heartbeat
        this.send({ type: 'heartbeat_response' });
        break;
        
      case 'auth_required':
        // Re-authenticate if token expired
        this.authenticateConnection();
        break;
        
      default:
        // Unknown event type
        break;
    }
  }
  
  // Start heartbeat to keep connection alive
  private startHeartbeat() {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      this.send({ type: 'ping' });
    }, this.heartbeatInterval);
  }
  
  // Stop heartbeat
  private stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
  
  // Schedule reconnection attempt
  private scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.log('Max reconnection attempts reached');
      return;
    }
    
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }
    
    const delay = this.reconnectInterval * Math.pow(2, this.reconnectAttempts);
    this.log(`Scheduling reconnect in ${delay}ms (attempt ${this.reconnectAttempts + 1})`);
    
    this.setState(WebSocketState.RECONNECTING);
    this.reconnectAttempts++;
    
    this.reconnectTimer = setTimeout(() => {
      this.connect().catch(error => {
        this.log('Reconnection failed:', error);
      });
    }, delay);
  }
  
  // Event subscription methods
  on(eventType: string, listener: WebSocketEventListener) {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType)!.add(listener);
    
    return () => this.off(eventType, listener);
  }
  
  off(eventType: string, listener: WebSocketEventListener) {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.delete(listener);
      if (listeners.size === 0) {
        this.eventListeners.delete(eventType);
      }
    }
  }
  
  // State subscription methods
  onStateChange(listener: (state: WebSocketState) => void) {
    this.stateListeners.add(listener);
    return () => this.stateListeners.delete(listener);
  }
  
  // Getters
  getState(): WebSocketState {
    return this.state;
  }
  
  isConnected(): boolean {
    return this.state === WebSocketState.CONNECTED;
  }
}

// Global WebSocket client instance
let globalWebSocketClient: WebSocketClient | null = null;

// Initialize WebSocket client
export function initializeWebSocketClient(config?: WebSocketClientConfig): WebSocketClient {
  if (globalWebSocketClient) {
    globalWebSocketClient.disconnect();
  }
  
  globalWebSocketClient = new WebSocketClient(config);
  return globalWebSocketClient;
}

// Get global WebSocket client
export function getWebSocketClient(): WebSocketClient {
  if (!globalWebSocketClient) {
    globalWebSocketClient = new WebSocketClient();
  }
  return globalWebSocketClient;
}

// React hook for WebSocket events
export function useWebSocketEvent(eventType: string, listener: WebSocketEventListener) {
  const client = getWebSocketClient();
  
  React.useEffect(() => {
    const unsubscribe = client.on(eventType, listener);
    return unsubscribe;
  }, [eventType, listener]);
}

// React hook for WebSocket state
export function useWebSocketState() {
  const client = getWebSocketClient();
  const [state, setState] = React.useState(client.getState());
  
  React.useEffect(() => {
    const unsubscribe = client.onStateChange(setState);
    return unsubscribe;
  }, []);
  
  return state;
}

// Auto-connect when authenticated
export function enableAutoConnect() {
  const client = getWebSocketClient();
  
  // Connect when session is available
  const checkAuthAndConnect = () => {
    const apiClient = getAPIClient();
    const sessionToken = apiClient.getSessionToken();
    
    if (sessionToken && !client.isConnected()) {
      client.connect().catch(error => {
        console.error('Failed to connect WebSocket:', error);
      });
    } else if (!sessionToken && client.isConnected()) {
      client.disconnect();
    }
  };
  
  // Check immediately
  checkAuthAndConnect();
  
  // Check periodically
  const interval = setInterval(checkAuthAndConnect, 5000);
  
  return () => clearInterval(interval);
}

// Import React for hooks (this would normally be imported at the top)
import React from 'react';