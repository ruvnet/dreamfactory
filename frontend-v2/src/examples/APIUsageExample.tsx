import React, { useState } from 'react';
import {
  useUsers,
  useCreateUser,
  useUpdateUser,
  useDeleteUser,
  useServices,
  useLogin,
  useSession,
  useLogout,
  User,
  LoginCredentials,
} from '../lib/api';

// Example: User Management Component
export function UserManagementExample() {
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [isCreating, setIsCreating] = useState(false);

  // Fetch users with React Query
  const {
    data: usersResponse,
    isLoading,
    error,
    refetch,
  } = useUsers({
    limit: 10,
    offset: 0,
    fields: 'id,name,email,is_active,created_date',
  });

  // Mutations
  const createUserMutation = useCreateUser({
    onSuccess: () => {
      setIsCreating(false);
      // Query will be automatically invalidated and refetched
    },
    onError: (error) => {
      console.error('Failed to create user:', error);
    },
  });

  const updateUserMutation = useUpdateUser({
    onSuccess: () => {
      setEditingUser(null);
    },
    onError: (error) => {
      console.error('Failed to update user:', error);
    },
  });

  const deleteUserMutation = useDeleteUser({
    onSuccess: () => {
      console.log('User deleted successfully');
    },
    onError: (error) => {
      console.error('Failed to delete user:', error);
    },
  });

  // Handlers
  const handleCreateUser = async (userData: Partial<User>) => {
    await createUserMutation.mutateAsync(userData);
  };

  const handleUpdateUser = async (id: number, userData: Partial<User>) => {
    await updateUserMutation.mutateAsync({ id, data: userData });
  };

  const handleDeleteUser = async (id: number) => {
    if (confirm('Are you sure you want to delete this user?')) {
      await deleteUserMutation.mutateAsync(id);
    }
  };

  if (isLoading) {
    return <div className="p-4">Loading users...</div>;
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded">
        <h3 className="text-red-800 font-semibold">Error loading users</h3>
        <p className="text-red-600">{error.error?.message || 'Unknown error'}</p>
        <button
          onClick={() => refetch()}
          className="mt-2 px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  const users = usersResponse?.resource || [];

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">User Management</h2>
        <button
          onClick={() => setIsCreating(true)}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Create User
        </button>
      </div>

      {/* Users Table */}
      <div className="overflow-x-auto">
        <table className="min-w-full bg-white border border-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Name
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Email
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Status
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Created
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {users.map((user) => (
              <tr key={user.id}>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm font-medium text-gray-900">{user.name}</div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <div className="text-sm text-gray-900">{user.email}</div>
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span
                    className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                      user.is_active
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {user.is_active ? 'Active' : 'Inactive'}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {new Date(user.created_date).toLocaleDateString()}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                  <button
                    onClick={() => setEditingUser(user)}
                    className="text-blue-600 hover:text-blue-900 mr-4"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDeleteUser(user.id)}
                    className="text-red-600 hover:text-red-900"
                    disabled={deleteUserMutation.isPending}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Create/Edit User Modal */}
      {(isCreating || editingUser) && (
        <UserModal
          user={editingUser}
          isOpen={isCreating || !!editingUser}
          onClose={() => {
            setIsCreating(false);
            setEditingUser(null);
          }}
          onSubmit={editingUser ? 
            (userData) => handleUpdateUser(editingUser.id, userData) :
            handleCreateUser
          }
          isLoading={createUserMutation.isPending || updateUserMutation.isPending}
        />
      )}
    </div>
  );
}

// Example: Authentication Component
export function AuthenticationExample() {
  const [credentials, setCredentials] = useState<LoginCredentials>({
    email: '',
    password: '',
  });

  // Check current session
  const { data: session, isLoading: sessionLoading } = useSession();

  // Login mutation
  const loginMutation = useLogin({
    onSuccess: (userData) => {
      console.log('Login successful:', userData);
    },
    onError: (error) => {
      console.error('Login failed:', error);
    },
  });

  // Logout mutation
  const logoutMutation = useLogout({
    onSuccess: () => {
      console.log('Logout successful');
    },
  });

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    await loginMutation.mutateAsync(credentials);
  };

  const handleLogout = async () => {
    await logoutMutation.mutateAsync({ isSysAdmin: session?.isSysAdmin });
  };

  if (sessionLoading) {
    return <div className="p-4">Checking authentication...</div>;
  }

  if (session) {
    return (
      <div className="p-6 bg-green-50 border border-green-200 rounded">
        <h3 className="text-lg font-semibold text-green-800 mb-2">
          Welcome, {session.name}!
        </h3>
        <p className="text-green-600 mb-4">
          You are logged in as {session.email}
          {session.isSysAdmin && ' (System Administrator)'}
        </p>
        <button
          onClick={handleLogout}
          disabled={logoutMutation.isPending}
          className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 disabled:opacity-50"
        >
          {logoutMutation.isPending ? 'Logging out...' : 'Logout'}
        </button>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-md mx-auto">
      <h2 className="text-2xl font-bold mb-6">Login</h2>
      <form onSubmit={handleLogin} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Email
          </label>
          <input
            type="email"
            value={credentials.email}
            onChange={(e) => setCredentials({ ...credentials, email: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            required
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Password
          </label>
          <input
            type="password"
            value={credentials.password}
            onChange={(e) => setCredentials({ ...credentials, password: e.target.value })}
            className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            required
          />
        </div>
        <button
          type="submit"
          disabled={loginMutation.isPending}
          className="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
        >
          {loginMutation.isPending ? 'Logging in...' : 'Login'}
        </button>
        {loginMutation.error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded">
            <p className="text-red-600 text-sm">
              {loginMutation.error.error?.message || 'Login failed'}
            </p>
          </div>
        )}
      </form>
    </div>
  );
}

// Example: Services Component
export function ServicesExample() {
  const { data: servicesResponse, isLoading, error } = useServices();

  if (isLoading) {
    return <div className="p-4">Loading services...</div>;
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded">
        <p className="text-red-600">{error.error?.message || 'Failed to load services'}</p>
      </div>
    );
  }

  const services = servicesResponse?.resource || [];

  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold mb-6">Services</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {services.map((service) => (
          <div key={service.id} className="bg-white border border-gray-200 rounded-lg p-4">
            <h3 className="text-lg font-semibold mb-2">{service.label}</h3>
            <p className="text-gray-600 text-sm mb-2">{service.description}</p>
            <div className="flex justify-between items-center">
              <span className="text-sm text-gray-500">Type: {service.type}</span>
              <span
                className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                  service.is_active
                    ? 'bg-green-100 text-green-800'
                    : 'bg-red-100 text-red-800'
                }`}
              >
                {service.is_active ? 'Active' : 'Inactive'}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// User Modal Component
interface UserModalProps {
  user?: User | null;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (userData: Partial<User>) => Promise<void>;
  isLoading: boolean;
}

function UserModal({ user, isOpen, onClose, onSubmit, isLoading }: UserModalProps) {
  const [formData, setFormData] = useState<Partial<User>>({
    name: user?.name || '',
    first_name: user?.first_name || '',
    last_name: user?.last_name || '',
    email: user?.email || '',
    is_active: user?.is_active ?? true,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSubmit(formData);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold mb-4">
          {user ? 'Edit User' : 'Create User'}
        </h3>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Name
            </label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              First Name
            </label>
            <input
              type="text"
              value={formData.first_name}
              onChange={(e) => setFormData({ ...formData, first_name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Last Name
            </label>
            <input
              type="text"
              value={formData.last_name}
              onChange={(e) => setFormData({ ...formData, last_name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Email
            </label>
            <input
              type="email"
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              required
            />
          </div>
          <div className="flex items-center">
            <input
              type="checkbox"
              id="is_active"
              checked={formData.is_active}
              onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
              className="mr-2"
            />
            <label htmlFor="is_active" className="text-sm font-medium text-gray-700">
              Active
            </label>
          </div>
          <div className="flex space-x-3 pt-4">
            <button
              type="submit"
              disabled={isLoading}
              className="flex-1 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
            >
              {isLoading ? 'Saving...' : user ? 'Update' : 'Create'}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 bg-gray-300 text-gray-700 rounded hover:bg-gray-400"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// Combined example component
export function APIIntegrationDemo() {
  const [activeTab, setActiveTab] = useState('auth');

  const tabs = [
    { id: 'auth', label: 'Authentication', component: AuthenticationExample },
    { id: 'users', label: 'User Management', component: UserManagementExample },
    { id: 'services', label: 'Services', component: ServicesExample },
  ];

  const ActiveComponent = tabs.find(tab => tab.id === activeTab)?.component || AuthenticationExample;

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex space-x-8">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`py-4 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </div>
      <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <ActiveComponent />
      </div>
    </div>
  );
}