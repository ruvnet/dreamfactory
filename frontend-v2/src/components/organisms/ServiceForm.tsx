import React, { useState, useEffect } from 'react';
import { Button } from '../atoms/Button';
import { Input } from '../atoms/Input';
import { Select } from '../atoms/Select';
import { Checkbox } from '../atoms/Checkbox';

interface ServiceFormData {
  name: string;
  label: string;
  description: string;
  type: string;
  is_active: boolean;
  config: Record<string, any>;
}

interface ServiceFormProps {
  initialData?: Partial<ServiceFormData>;
  onSubmit: (data: ServiceFormData) => void;
  onCancel: () => void;
  isSubmitting?: boolean;
}

const SERVICE_TYPES = [
  { value: 'database', label: 'Database' },
  { value: 'file', label: 'File Storage' },
  { value: 'email', label: 'Email Service' },
  { value: 'remote_web', label: 'Remote Web Service' },
  { value: 'script', label: 'Script Service' },
  { value: 'oauth', label: 'OAuth Service' },
  { value: 'user', label: 'User Service' },
  { value: 'system', label: 'System Service' },
];

export const ServiceForm: React.FC<ServiceFormProps> = ({
  initialData,
  onSubmit,
  onCancel,
  isSubmitting = false,
}) => {
  const [formData, setFormData] = useState<ServiceFormData>({
    name: '',
    label: '',
    description: '',
    type: 'database',
    is_active: true,
    config: {},
    ...initialData,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (initialData) {
      setFormData({
        name: '',
        label: '',
        description: '',
        type: 'database',
        is_active: true,
        config: {},
        ...initialData,
      });
    }
  }, [initialData]);

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Service name is required';
    } else if (!/^[a-zA-Z0-9_]+$/.test(formData.name)) {
      newErrors.name = 'Service name can only contain letters, numbers, and underscores';
    }

    if (!formData.label.trim()) {
      newErrors.label = 'Service label is required';
    }

    if (!formData.type) {
      newErrors.type = 'Service type is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validateForm()) {
      onSubmit(formData);
    }
  };

  const handleInputChange = (field: keyof ServiceFormData, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }));
    }
  };

  const renderTypeSpecificFields = () => {
    switch (formData.type) {
      case 'database':
        return (
          <div className="space-y-4">
            <h4 className="font-medium text-gray-900">Database Configuration</h4>
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Host"
                value={formData.config.host || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, host: e.target.value })}
                placeholder="localhost"
              />
              <Input
                label="Port"
                type="number"
                value={formData.config.port || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, port: e.target.value })}
                placeholder="3306"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Database Name"
                value={formData.config.database || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, database: e.target.value })}
                placeholder="my_database"
              />
              <Select
                label="Driver"
                value={formData.config.driver || 'mysql'}
                onChange={(value) => handleInputChange('config', { ...formData.config, driver: value })}
                options={[
                  { value: 'mysql', label: 'MySQL' },
                  { value: 'pgsql', label: 'PostgreSQL' },
                  { value: 'sqlite', label: 'SQLite' },
                  { value: 'sqlsrv', label: 'SQL Server' },
                ]}
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Username"
                value={formData.config.username || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, username: e.target.value })}
                placeholder="db_user"
              />
              <Input
                label="Password"
                type="password"
                value={formData.config.password || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, password: e.target.value })}
                placeholder="••••••••"
              />
            </div>
          </div>
        );
      
      case 'file':
        return (
          <div className="space-y-4">
            <h4 className="font-medium text-gray-900">File Storage Configuration</h4>
            <Select
              label="Storage Type"
              value={formData.config.type || 'local'}
              onChange={(value) => handleInputChange('config', { ...formData.config, type: value })}
              options={[
                { value: 'local', label: 'Local File System' },
                { value: 's3', label: 'Amazon S3' },
                { value: 'azure', label: 'Azure Blob Storage' },
                { value: 'gcs', label: 'Google Cloud Storage' },
              ]}
            />
            {formData.config.type === 'local' && (
              <Input
                label="Root Path"
                value={formData.config.root || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, root: e.target.value })}
                placeholder="/var/www/storage"
              />
            )}
          </div>
        );
      
      case 'email':
        return (
          <div className="space-y-4">
            <h4 className="font-medium text-gray-900">Email Service Configuration</h4>
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="SMTP Host"
                value={formData.config.host || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, host: e.target.value })}
                placeholder="smtp.gmail.com"
              />
              <Input
                label="Port"
                type="number"
                value={formData.config.port || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, port: e.target.value })}
                placeholder="587"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Username"
                value={formData.config.username || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, username: e.target.value })}
                placeholder="your-email@gmail.com"
              />
              <Input
                label="Password"
                type="password"
                value={formData.config.password || ''}
                onChange={(e) => handleInputChange('config', { ...formData.config, password: e.target.value })}
                placeholder="••••••••"
              />
            </div>
            <Checkbox
              label="Use TLS Encryption"
              checked={formData.config.encryption === 'tls'}
              onChange={(checked) => handleInputChange('config', { 
                ...formData.config, 
                encryption: checked ? 'tls' : 'none' 
              })}
            />
          </div>
        );
      
      default:
        return (
          <div className="text-sm text-gray-500">
            Additional configuration options will appear here based on the service type.
          </div>
        );
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Basic Information */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Basic Information</h3>
        
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="Service Name"
            value={formData.name}
            onChange={(e) => handleInputChange('name', e.target.value)}
            error={errors.name}
            placeholder="my_service"
            disabled={!!initialData?.name} // Disable editing name for existing services
            helperText="Used in API endpoints (lowercase, underscores only)"
          />
          
          <Input
            label="Display Label"
            value={formData.label}
            onChange={(e) => handleInputChange('label', e.target.value)}
            error={errors.label}
            placeholder="My Service"
            helperText="Human-readable service name"
          />
        </div>

        <Input
          label="Description"
          value={formData.description}
          onChange={(e) => handleInputChange('description', e.target.value)}
          placeholder="Brief description of this service"
          helperText="Optional description for documentation"
        />

        <Select
          label="Service Type"
          value={formData.type}
          onChange={(value) => handleInputChange('type', value)}
          options={SERVICE_TYPES}
          error={errors.type}
          helperText="Choose the type of service to create"
        />

        <Checkbox
          label="Active"
          checked={formData.is_active}
          onChange={(checked) => handleInputChange('is_active', checked)}
          helperText="Whether this service is available for API calls"
        />
      </div>

      {/* Type-specific configuration */}
      <div className="border-t border-gray-200 pt-6">
        {renderTypeSpecificFields()}
      </div>

      {/* Form Actions */}
      <div className="flex justify-end space-x-3 pt-6 border-t border-gray-200">
        <Button
          type="button"
          variant="outline"
          onClick={onCancel}
          disabled={isSubmitting}
        >
          Cancel
        </Button>
        <Button
          type="submit"
          variant="primary"
          loading={isSubmitting}
          disabled={isSubmitting}
        >
          {initialData ? 'Update Service' : 'Create Service'}
        </Button>
      </div>
    </form>
  );
};

export default ServiceForm;