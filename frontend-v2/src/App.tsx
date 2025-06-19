import { DreamFactoryProvider } from './providers/APIProvider';
import { APIIntegrationDemo } from './examples/APIUsageExample';

// API configuration
const apiConfig = {
  baseURL: '/api/v2',
  apiKey: import.meta.env['VITE_DREAMFACTORY_API_KEY'] || 'demo-api-key',
  enableMocks: import.meta.env.DEV, // Enable mocks in development
  enableWebSocket: true,
  debug: import.meta.env.DEV,
};

function App() {
  return (
    <DreamFactoryProvider config={apiConfig}>
      <div className="min-h-screen bg-gray-50">
        <header className="bg-white shadow-sm border-b">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
            <div className="flex items-center justify-between">
              <h1 className="text-2xl font-bold text-gray-900">
                DreamFactory API Integration Demo
              </h1>
              <div className="text-sm text-gray-500">
                React Query + TypeScript + WebSocket
              </div>
            </div>
          </div>
        </header>
        
        <main>
          <APIIntegrationDemo />
        </main>
        
        <footer className="bg-white border-t mt-auto">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
            <div className="text-center text-sm text-gray-500">
              <p>
                Comprehensive API integration with React Query, Zod validation, 
                WebSocket real-time updates, and MSW mocking
              </p>
              <div className="mt-2 space-x-4">
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-green-100 text-green-800">
                  ✅ React Query Caching
                </span>
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800">
                  ✅ TypeScript Validation
                </span>
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-purple-100 text-purple-800">
                  ✅ WebSocket Integration
                </span>
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-yellow-100 text-yellow-800">
                  ✅ API Mocking
                </span>
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-red-100 text-red-800">
                  ✅ Error Handling
                </span>
              </div>
            </div>
          </div>
        </footer>
      </div>
    </DreamFactoryProvider>
  );
}

export default App;