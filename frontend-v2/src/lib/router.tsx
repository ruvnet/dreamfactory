import { createBrowserRouter, Navigate } from 'react-router-dom'
import { lazy, Suspense } from 'react'

// Layout components
import { AuthLayout } from '@/components/templates/AuthLayout/AuthLayout'
import { DashboardLayout } from '@/components/templates/DashboardLayout/DashboardLayout'

// Loading component
const PageLoader = () => (
  <div className="flex items-center justify-center min-h-screen">
    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
  </div>
)

// Lazy load page components
const LoginPage = lazy(() => import('@/pages/auth/login'))
const RegisterPage = lazy(() => import('@/pages/auth/register'))
const ForgotPasswordPage = lazy(() => import('@/pages/auth/forgot-password'))

const DashboardPage = lazy(() => import('@/pages/dashboard'))
const ServicesPage = lazy(() => import('@/pages/services'))
const UsersPage = lazy(() => import('@/pages/users'))
const RolesPage = lazy(() => import('@/pages/roles'))
const SettingsPage = lazy(() => import('@/pages/settings'))
const ProfilePage = lazy(() => import('@/pages/profile'))

// Route guard component
const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
  // Check for session token in localStorage
  const sessionToken = localStorage.getItem('df_session_token')
  const isAuthenticated = !!sessionToken
  
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }
  
  return <>{children}</>
}

// Public route component (redirect if authenticated)
const PublicRoute = ({ children }: { children: React.ReactNode }) => {
  // Check for session token in localStorage
  const sessionToken = localStorage.getItem('df_session_token')
  const isAuthenticated = !!sessionToken
  
  if (isAuthenticated) {
    return <Navigate to="/dashboard" replace />
  }
  
  return <>{children}</>
}

// Lazy wrapper with suspense
const LazyWrapper = ({ Component }: { Component: React.LazyExoticComponent<any> }) => (
  <Suspense fallback={<PageLoader />}>
    <Component />
  </Suspense>
)

export const router = createBrowserRouter([
  {
    path: '/',
    element: <Navigate to="/dashboard" replace />,
  },
  {
    path: '/login',
    element: (
      <PublicRoute>
        <AuthLayout>
          <LazyWrapper Component={LoginPage} />
        </AuthLayout>
      </PublicRoute>
    ),
  },
  {
    path: '/register',
    element: (
      <PublicRoute>
        <AuthLayout>
          <LazyWrapper Component={RegisterPage} />
        </AuthLayout>
      </PublicRoute>
    ),
  },
  {
    path: '/forgot-password',
    element: (
      <PublicRoute>
        <AuthLayout>
          <LazyWrapper Component={ForgotPasswordPage} />
        </AuthLayout>
      </PublicRoute>
    ),
  },
  {
    path: '/dashboard',
    element: (
      <ProtectedRoute>
        <DashboardLayout>
          <LazyWrapper Component={DashboardPage} />
        </DashboardLayout>
      </ProtectedRoute>
    ),
  },
  {
    path: '/services',
    element: (
      <ProtectedRoute>
        <DashboardLayout>
          <LazyWrapper Component={ServicesPage} />
        </DashboardLayout>
      </ProtectedRoute>
    ),
  },
  {
    path: '/users',
    element: (
      <ProtectedRoute>
        <DashboardLayout>
          <LazyWrapper Component={UsersPage} />
        </DashboardLayout>
      </ProtectedRoute>
    ),
  },
  {
    path: '/roles',
    element: (
      <ProtectedRoute>
        <DashboardLayout>
          <LazyWrapper Component={RolesPage} />
        </DashboardLayout>
      </ProtectedRoute>
    ),
  },
  {
    path: '/settings',
    element: (
      <ProtectedRoute>
        <DashboardLayout>
          <LazyWrapper Component={SettingsPage} />
        </DashboardLayout>
      </ProtectedRoute>
    ),
  },
  {
    path: '/profile',
    element: (
      <ProtectedRoute>
        <DashboardLayout>
          <LazyWrapper Component={ProfilePage} />
        </DashboardLayout>
      </ProtectedRoute>
    ),
  },
  {
    path: '*',
    element: (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-4xl font-bold text-gray-900 mb-4">404</h1>
          <p className="text-gray-600 mb-8">Page not found</p>
          <Navigate to="/dashboard" replace />
        </div>
      </div>
    ),
  },
])