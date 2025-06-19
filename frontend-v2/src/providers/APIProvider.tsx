import React, { createContext, useContext, useEffect, useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { queryClient } from '../lib/api/queryClient';
import { initializeDreamFactoryAPI } from '../lib/api';

// API configuration interface
interface APIConfig {
  baseURL?: string;
  apiKey: string;
  enableMocks?: boolean;
  enableWebSocket?: boolean;
  debug?: boolean;
}

// Context interface
interface APIContextValue {
  isInitialized: boolean;
  error: Error | null;
  config: APIConfig;
  reinitialize: (newConfig: APIConfig) => Promise<void>;
}

// Create context
const APIContext = createContext<APIContextValue | null>(null);

// Hook to use API context
export function useAPI() {
  const context = useContext(APIContext);
  if (!context) {
    throw new Error('useAPI must be used within an APIProvider');
  }
  return context;
}

// Provider component
interface APIProviderProps {
  children: React.ReactNode;
  config: APIConfig;
  queryClient?: QueryClient;
}

export function APIProvider({ children, config, queryClient: customQueryClient }: APIProviderProps) {
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [currentConfig, setCurrentConfig] = useState(config);

  // Use custom query client or default
  const client = customQueryClient || queryClient;

  // Initialize API
  const initialize = async (apiConfig: APIConfig) => {
    try {
      setError(null);
      setIsInitialized(false);

      await initializeDreamFactoryAPI(apiConfig);
      
      setCurrentConfig(apiConfig);
      setIsInitialized(true);
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to initialize API');
      setError(error);
      console.error('API initialization failed:', error);
    }
  };

  // Reinitialize with new config
  const reinitialize = async (newConfig: APIConfig) => {
    await initialize(newConfig);
  };

  // Initialize on mount
  useEffect(() => {
    initialize(config);
  }, []); // Only run once on mount

  // Context value
  const contextValue: APIContextValue = {
    isInitialized,
    error,
    config: currentConfig,
    reinitialize,
  };

  return (
    <APIContext.Provider value={contextValue}>
      <QueryClientProvider client={client}>
        {children}
        {process.env.NODE_ENV === 'development' && (
          <ReactQueryDevtools initialIsOpen={false} />
        )}
      </QueryClientProvider>
    </APIContext.Provider>
  );
}

// Error boundary for API errors
interface APIErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class APIErrorBoundary extends React.Component<
  { children: React.ReactNode; fallback?: React.ComponentType<{ error: Error }> },
  APIErrorBoundaryState
> {
  constructor(props: any) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): APIErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('API Error Boundary caught an error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError && this.state.error) {
      if (this.props.fallback) {
        const FallbackComponent = this.props.fallback;
        return <FallbackComponent error={this.state.error} />;
      }

      return (
        <div className="flex items-center justify-center min-h-screen">
          <div className="text-center">
            <h2 className="text-2xl font-bold text-red-600 mb-4">
              API Error
            </h2>
            <p className="text-gray-600 mb-4">
              {this.state.error.message}
            </p>
            <button
              onClick={() => this.setState({ hasError: false, error: null })}
              className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
            >
              Try Again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

// Higher-order component for API error boundary
export function withAPIErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  fallback?: React.ComponentType<{ error: Error }>
) {
  return function WrappedComponent(props: P) {
    return (
      <APIErrorBoundary fallback={fallback}>
        <Component {...props} />
      </APIErrorBoundary>
    );
  };
}

// Loading component for API initialization
export function APILoadingScreen() {
  return (
    <div className="flex items-center justify-center min-h-screen">
      <div className="text-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-500 mx-auto mb-4"></div>
        <h2 className="text-xl font-semibold text-gray-700 mb-2">
          Initializing API
        </h2>
        <p className="text-gray-500">
          Please wait while we set up the connection...
        </p>
      </div>
    </div>
  );
}

// Combined provider with error boundary and loading
interface DreamFactoryProviderProps {
  children: React.ReactNode;
  config: APIConfig;
  loadingComponent?: React.ComponentType;
  errorComponent?: React.ComponentType<{ error: Error }>;
  queryClient?: QueryClient;
}

export function DreamFactoryProvider({
  children,
  config,
  loadingComponent: LoadingComponent = APILoadingScreen,
  errorComponent,
  queryClient: customQueryClient,
}: DreamFactoryProviderProps) {
  return (
    <APIErrorBoundary fallback={errorComponent}>
      <APIProvider config={config} queryClient={customQueryClient}>
        <APIInitializationGuard loadingComponent={LoadingComponent}>
          {children}
        </APIInitializationGuard>
      </APIProvider>
    </APIErrorBoundary>
  );
}

// Guard component that waits for API initialization
function APIInitializationGuard({ 
  children, 
  loadingComponent: LoadingComponent = APILoadingScreen 
}: { 
  children: React.ReactNode;
  loadingComponent?: React.ComponentType;
}) {
  const { isInitialized, error } = useAPI();

  if (error) {
    throw error; // Will be caught by error boundary
  }

  if (!isInitialized) {
    return <LoadingComponent />;
  }

  return <>{children}</>;
}

export default APIProvider;