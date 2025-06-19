import React from 'react'

interface DashboardLayoutProps {
  children: React.ReactNode
}

export const DashboardLayout: React.FC<DashboardLayoutProps> = ({ children }) => {
  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <div className="flex items-center">
              <h1 className="text-2xl font-bold text-primary-600">DreamFactory</h1>
            </div>
            
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-700">
                Welcome back, {JSON.parse(localStorage.getItem('df_user') || '{"name":"User"}').name}!
              </span>
              <button
                onClick={() => {
                  localStorage.removeItem('df_session_token');
                  localStorage.removeItem('df_user');
                  window.location.href = '/login';
                }}
                className="text-sm text-gray-500 hover:text-gray-700"
              >
                Logout
              </button>
              <div className="w-8 h-8 bg-primary-600 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-medium">
                  {JSON.parse(localStorage.getItem('df_user') || '{"name":"U"}').name.charAt(0).toUpperCase()}
                </span>
              </div>
            </div>
          </div>
        </div>
      </header>
      
      <div className="flex">
        {/* Sidebar */}
        <aside className="w-64 bg-white shadow-sm h-[calc(100vh-80px)]">
          <nav className="mt-8">
            <div className="px-3">
              <div className="space-y-1">
                <a href="/dashboard" className="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100">
                  Dashboard
                </a>
                <a href="/services" className="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100">
                  Services
                </a>
                <a href="/users" className="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100">
                  Users
                </a>
                <a href="/roles" className="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100">
                  Roles
                </a>
                <a href="/settings" className="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100">
                  Settings
                </a>
              </div>
            </div>
          </nav>
        </aside>
        
        {/* Main content */}
        <main className="flex-1 p-6">
          {children}
        </main>
      </div>
    </div>
  )
}