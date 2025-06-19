import { Navigate } from 'react-router-dom';
import { useIsAuthenticated } from '../../lib/api/services/auth';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requireAdmin?: boolean;
}

export function ProtectedRoute({ children, requireAdmin = false }: ProtectedRouteProps) {
  const isAuthenticated = useIsAuthenticated();
  
  if (!isAuthenticated) {
    return <Navigate to="/auth/login" replace />;
  }
  
  // TODO: Add admin check when requireAdmin is true
  // if (requireAdmin && !isAdmin) {
  //   return <Navigate to="/" replace />;
  // }
  
  return <>{children}</>;
}