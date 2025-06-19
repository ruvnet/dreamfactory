import React from 'react';
import { Card } from '../../components/molecules/Card';

export const RolesPage: React.FC = () => {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Roles & Permissions</h1>
        <p className="text-gray-600 mt-1">
          Manage user roles and permissions
        </p>
      </div>

      <Card className="p-8 text-center">
        <div className="text-gray-500">
          <div className="text-lg font-medium mb-2">Roles Management</div>
          <div>Role and permission management interface coming soon...</div>
        </div>
      </Card>
    </div>
  );
};

export default RolesPage;