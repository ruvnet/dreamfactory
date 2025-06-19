import React from 'react';
import { Card } from '../../components/molecules/Card';

export const ProfilePage: React.FC = () => {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">User Profile</h1>
        <p className="text-gray-600 mt-1">
          Manage your profile and account settings
        </p>
      </div>

      <Card className="p-8 text-center">
        <div className="text-gray-500">
          <div className="text-lg font-medium mb-2">Profile Management</div>
          <div>User profile interface coming soon...</div>
        </div>
      </Card>
    </div>
  );
};

export default ProfilePage;