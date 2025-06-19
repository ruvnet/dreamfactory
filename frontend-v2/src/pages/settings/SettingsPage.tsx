import React from 'react';
import { Card } from '../../components/molecules/Card';

export const SettingsPage: React.FC = () => {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">System Settings</h1>
        <p className="text-gray-600 mt-1">
          Configure system-wide settings and preferences
        </p>
      </div>

      <Card className="p-8 text-center">
        <div className="text-gray-500">
          <div className="text-lg font-medium mb-2">System Configuration</div>
          <div>System settings interface coming soon...</div>
        </div>
      </Card>
    </div>
  );
};

export default SettingsPage;