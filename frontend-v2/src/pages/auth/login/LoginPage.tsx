import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '../../../components/atoms/Button';
import { Input } from '../../../components/atoms/Input';
import { Card } from '../../../components/molecules/Card';

const LoginPage: React.FC = () => {
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    email: '',
    password: '',
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError('');

    try {
      // For demo purposes, accept any email/password and create a fake session
      if (formData.email && formData.password) {
        // Simulate API call delay
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Store fake session token
        const fakeToken = btoa(JSON.stringify({
          email: formData.email,
          timestamp: Date.now(),
          expires: Date.now() + (24 * 60 * 60 * 1000) // 24 hours
        }));
        
        localStorage.setItem('df_session_token', fakeToken);
        localStorage.setItem('df_user', JSON.stringify({
          email: formData.email,
          name: formData.email.split('@')[0],
          is_sys_admin: true
        }));
        
        // Redirect to dashboard
        navigate('/dashboard');
      } else {
        setError('Please enter both email and password');
      }
    } catch (err) {
      setError('Login failed. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (field: keyof typeof formData, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (error) setError('');
  };

  // Demo credentials for easy testing
  const demoCredentials = [
    { email: 'admin@dreamfactory.com', password: 'admin123', role: 'System Admin' },
    { email: 'user@dreamfactory.com', password: 'user123', role: 'User' },
    { email: 'demo@example.com', password: 'demo123', role: 'Demo User' },
  ];

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        {/* Header */}
        <div>
          <div className="mx-auto h-12 w-12 flex items-center justify-center rounded-full bg-blue-100">
            <svg className="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Sign in to DreamFactory
          </h2>
          <p className="mt-2 text-center text-sm text-gray-600">
            Manage your API platform
          </p>
        </div>

        {/* Demo Credentials Info */}
        <Card className="p-4 bg-blue-50 border-blue-200">
          <h3 className="text-sm font-medium text-blue-800 mb-2">Demo Credentials:</h3>
          <div className="space-y-1">
            {demoCredentials.map((cred, index) => (
              <div key={index} className="text-xs text-blue-700">
                <span className="font-mono">{cred.email}</span> / <span className="font-mono">{cred.password}</span>
                <span className="text-blue-600 ml-2">({cred.role})</span>
              </div>
            ))}
          </div>
          <p className="text-xs text-blue-600 mt-2">
            Any email/password combination will work for this demo
          </p>
        </Card>

        {/* Login Form */}
        <Card className="p-8">
          <form className="space-y-6" onSubmit={handleSubmit}>
            {error && (
              <div className="bg-red-50 border border-red-200 rounded-md p-3">
                <div className="text-sm text-red-600">{error}</div>
              </div>
            )}

            <Input
              label="Email address"
              type="email"
              value={formData.email}
              onChange={(e) => handleInputChange('email', e.target.value)}
              placeholder="admin@dreamfactory.com"
              required
              autoComplete="email"
            />

            <Input
              label="Password"
              type="password"
              value={formData.password}
              onChange={(e) => handleInputChange('password', e.target.value)}
              placeholder="admin123"
              required
              autoComplete="current-password"
            />

            <Button
              type="submit"
              variant="primary"
              className="w-full"
              loading={isLoading}
              disabled={isLoading}
            >
              {isLoading ? 'Signing in...' : 'Sign in'}
            </Button>
          </form>

          <div className="mt-6">
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-300" />
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-2 bg-white text-gray-500">Quick Login</span>
              </div>
            </div>

            <div className="mt-4 grid grid-cols-1 gap-2">
              {demoCredentials.map((cred, index) => (
                <Button
                  key={index}
                  variant="outline"
                  size="sm"
                  className="text-xs"
                  onClick={async () => {
                    setFormData({ email: cred.email, password: cred.password });
                    setIsLoading(true);
                    setError('');
                    
                    // Simulate API call delay
                    await new Promise(resolve => setTimeout(resolve, 500));
                    
                    // Store fake session token
                    const fakeToken = btoa(JSON.stringify({
                      email: cred.email,
                      timestamp: Date.now(),
                      expires: Date.now() + (24 * 60 * 60 * 1000) // 24 hours
                    }));
                    
                    localStorage.setItem('df_session_token', fakeToken);
                    localStorage.setItem('df_user', JSON.stringify({
                      email: cred.email,
                      name: cred.email.split('@')[0],
                      is_sys_admin: cred.role === 'System Admin'
                    }));
                    
                    // Redirect to dashboard
                    navigate('/dashboard');
                  }}
                  disabled={isLoading}
                >
                  Login as {cred.role}
                </Button>
              ))}
            </div>
          </div>
        </Card>

        {/* Footer */}
        <div className="text-center">
          <p className="text-xs text-gray-500">
            DreamFactory Admin Interface Demo
          </p>
        </div>
      </div>
    </div>
  );
};

export default LoginPage;