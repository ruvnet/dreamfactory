import React, { useState } from 'react';
import { PlusIcon, UserIcon, TrashIcon, EyeIcon, PencilIcon } from '@heroicons/react/24/outline';
import { Button } from '../../components/atoms/Button';
import { Input } from '../../components/atoms/Input';
import { Badge } from '../../components/atoms/Badge';
import { Card } from '../../components/molecules/Card';
import { Modal } from '../../components/molecules/Modal';
import { DataTable } from '../../components/organisms/DataTable';
import { UserForm } from '../../components/organisms/UserForm';
import { useUsers, useCreateUser, useUpdateUser, useDeleteUser } from '../../lib/api/services/users';

const UserStatusBadge: React.FC<{ status: boolean }> = ({ status }) => {
  return (
    <Badge variant={status ? 'success' : 'error'}>
      {status ? 'Active' : 'Inactive'}
    </Badge>
  );
};

const UserRoleBadge: React.FC<{ roles: string[] }> = ({ roles }) => {
  if (!roles || roles.length === 0) {
    return <Badge variant="gray">No Roles</Badge>;
  }
  
  if (roles.length === 1) {
    return <Badge variant="blue">{roles[0]}</Badge>;
  }
  
  return (
    <div className="flex space-x-1">
      <Badge variant="blue">{roles[0]}</Badge>
      {roles.length > 1 && (
        <Badge variant="gray">+{roles.length - 1}</Badge>
      )}
    </div>
  );
};

export const UsersPage: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<any>(null);
  
  const { data: usersResponse, isLoading, error } = useUsers();
  const createUser = useCreateUser();
  const updateUser = useUpdateUser();
  const deleteUser = useDeleteUser();
  
  const users = usersResponse?.resource || [];

  const columns = [
    {
      key: 'name',
      header: 'User',
      sortable: true,
      render: (user: any) => (
        <div className="flex items-center space-x-3">
          <div className="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
            <UserIcon className="w-4 h-4 text-blue-600" />
          </div>
          <div>
            <div className="font-medium text-gray-900">
              {user.first_name && user.last_name 
                ? `${user.first_name} ${user.last_name}`
                : user.name || user.email
              }
            </div>
            <div className="text-sm text-gray-500">{user.email}</div>
          </div>
        </div>
      ),
    },
    {
      key: 'username',
      header: 'Username',
      sortable: true,
      render: (user: any) => (
        <span className="text-gray-900">{user.username || '-'}</span>
      ),
    },
    {
      key: 'roles',
      header: 'Roles',
      render: (user: any) => <UserRoleBadge roles={user.role_names || []} />,
    },
    {
      key: 'is_active',
      header: 'Status',
      sortable: true,
      render: (user: any) => <UserStatusBadge status={user.is_active} />,
    },
    {
      key: 'last_login_date',
      header: 'Last Login',
      sortable: true,
      render: (user: any) => (
        user.last_login_date 
          ? new Date(user.last_login_date).toLocaleDateString()
          : 'Never'
      ),
    },
    {
      key: 'created_date',
      header: 'Created',
      sortable: true,
      render: (user: any) => new Date(user.created_date).toLocaleDateString(),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (user: any) => (
        <div className="flex space-x-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleViewUser(user)}
            className="text-blue-600 hover:text-blue-700"
          >
            <EyeIcon className="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleEditUser(user)}
            className="text-gray-600 hover:text-gray-700"
          >
            <PencilIcon className="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleDeleteUser(user)}
            className="text-red-600 hover:text-red-700"
            disabled={user.is_sys_admin} // Prevent deleting system admin
          >
            <TrashIcon className="w-4 h-4" />
          </Button>
        </div>
      ),
    },
  ];

  const handleCreateUser = async (userData: any) => {
    try {
      await createUser.mutateAsync(userData);
      setIsCreateModalOpen(false);
    } catch (error) {
      console.error('Failed to create user:', error);
    }
  };

  const handleEditUser = (user: any) => {
    setSelectedUser(user);
    setIsEditModalOpen(true);
  };

  const handleUpdateUser = async (userData: any) => {
    try {
      await updateUser.mutateAsync({ id: selectedUser.id, data: userData });
      setIsEditModalOpen(false);
      setSelectedUser(null);
    } catch (error) {
      console.error('Failed to update user:', error);
    }
  };

  const handleDeleteUser = async (user: any) => {
    if (user.is_sys_admin) {
      alert('Cannot delete system administrator user.');
      return;
    }
    
    if (window.confirm(`Are you sure you want to delete the user "${user.email}"?`)) {
      try {
        await deleteUser.mutateAsync(user.id);
      } catch (error) {
        console.error('Failed to delete user:', error);
      }
    }
  };

  const handleViewUser = (user: any) => {
    // Navigate to user details page or show details modal
    console.log('View user:', user);
  };

  const filteredUsers = users?.filter(user =>
    user.email?.toLowerCase().includes(searchTerm.toLowerCase()) ||
    user.username?.toLowerCase().includes(searchTerm.toLowerCase()) ||
    user.first_name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
    user.last_name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
    user.role_names?.some((role: string) => 
      role.toLowerCase().includes(searchTerm.toLowerCase())
    )
  ) || [];

  if (error) {
    return (
      <div className="p-6">
        <Card className="p-8 text-center">
          <div className="text-red-600 text-lg font-medium mb-2">Error loading users</div>
          <div className="text-gray-600">{error.message}</div>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Users</h1>
          <p className="text-gray-600 mt-1">
            Manage user accounts and permissions
          </p>
        </div>
        <Button
          variant="primary"
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center space-x-2"
        >
          <PlusIcon className="w-4 h-4" />
          <span>Create User</span>
        </Button>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="p-4">
          <div className="text-sm font-medium text-gray-500">Total Users</div>
          <div className="text-2xl font-bold text-gray-900">{users.length}</div>
        </Card>
        <Card className="p-4">
          <div className="text-sm font-medium text-gray-500">Active Users</div>
          <div className="text-2xl font-bold text-green-600">
            {users.filter(u => u.is_active).length}
          </div>
        </Card>
        <Card className="p-4">
          <div className="text-sm font-medium text-gray-500">Inactive Users</div>
          <div className="text-2xl font-bold text-red-600">
            {users.filter(u => !u.is_active).length}
          </div>
        </Card>
        <Card className="p-4">
          <div className="text-sm font-medium text-gray-500">System Admins</div>
          <div className="text-2xl font-bold text-blue-600">
            {users.filter(u => u.is_sys_admin).length}
          </div>
        </Card>
      </div>

      {/* Search and Filters */}
      <Card className="p-4">
        <div className="flex space-x-4">
          <div className="flex-1">
            <Input
              type="text"
              placeholder="Search users by name, email, username, or role..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full"
            />
          </div>
        </div>
      </Card>

      {/* Users Table */}
      <Card>
        <DataTable
          data={filteredUsers}
          columns={columns}
          loading={isLoading}
          emptyMessage="No users found. Create your first user to get started."
        />
      </Card>

      {/* Create User Modal */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="Create New User"
        size="lg"
      >
        <UserForm
          onSubmit={handleCreateUser}
          onCancel={() => setIsCreateModalOpen(false)}
          isSubmitting={createUser.isPending}
        />
      </Modal>

      {/* Edit User Modal */}
      <Modal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        title="Edit User"
        size="lg"
      >
        {selectedUser && (
          <UserForm
            initialData={selectedUser}
            onSubmit={handleUpdateUser}
            onCancel={() => setIsEditModalOpen(false)}
            isSubmitting={updateUser.isPending}
          />
        )}
      </Modal>
    </div>
  );
};

export default UsersPage;