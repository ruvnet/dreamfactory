import React, { useState, useEffect } from 'react';
import { Button } from '../atoms/Button';
import { Input } from '../atoms/Input';
import { Select } from '../atoms/Select';
import { Checkbox } from '../atoms/Checkbox';

interface UserFormData {
  email: string;
  username?: string;
  first_name: string;
  last_name: string;
  password?: string;
  confirm_password?: string;
  is_active: boolean;
  is_sys_admin: boolean;
  role_ids?: number[];
  phone?: string;
  security_question?: string;
  security_answer?: string;
}

interface UserFormProps {
  initialData?: Partial<UserFormData>;
  onSubmit: (data: UserFormData) => void;
  onCancel: () => void;
  isSubmitting?: boolean;
}

export const UserForm: React.FC<UserFormProps> = ({
  initialData,
  onSubmit,
  onCancel,
  isSubmitting = false,
}) => {
  const [formData, setFormData] = useState<UserFormData>({
    email: '',
    username: '',
    first_name: '',
    last_name: '',
    password: '',
    confirm_password: '',
    is_active: true,
    is_sys_admin: false,
    role_ids: [],
    phone: '',
    security_question: '',
    security_answer: '',
    ...initialData,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [showPasswordFields, setShowPasswordFields] = useState(!initialData);

  useEffect(() => {
    if (initialData) {
      setFormData({
        email: '',
        username: '',
        first_name: '',
        last_name: '',
        password: '',
        confirm_password: '',
        is_active: true,
        is_sys_admin: false,
        role_ids: [],
        phone: '',
        security_question: '',
        security_answer: '',
        ...initialData,
      });
    }
  }, [initialData]);

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    // Email validation
    if (!formData.email.trim()) {
      newErrors.email = 'Email is required';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      newErrors.email = 'Please enter a valid email address';
    }

    // Name validation
    if (!formData.first_name.trim()) {
      newErrors.first_name = 'First name is required';
    }

    if (!formData.last_name.trim()) {
      newErrors.last_name = 'Last name is required';
    }

    // Password validation for new users or when changing password
    if (showPasswordFields) {
      if (!formData.password) {
        newErrors.password = 'Password is required';
      } else if (formData.password.length < 8) {
        newErrors.password = 'Password must be at least 8 characters long';
      }

      if (!formData.confirm_password) {
        newErrors.confirm_password = 'Please confirm your password';
      } else if (formData.password !== formData.confirm_password) {
        newErrors.confirm_password = 'Passwords do not match';
      }
    }

    // Username validation (optional but must be unique if provided)
    if (formData.username && formData.username.length < 3) {
      newErrors.username = 'Username must be at least 3 characters long';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validateForm()) {
      const submitData = { ...formData };
      
      // Remove password fields if not shown (for edit mode)
      if (!showPasswordFields) {
        delete submitData.password;
        delete submitData.confirm_password;
      } else {
        // Remove confirm_password from submission
        delete submitData.confirm_password;
      }
      
      onSubmit(submitData);
    }
  };

  const handleInputChange = (field: keyof UserFormData, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }));
    }
  };

  const SECURITY_QUESTIONS = [
    { value: '', label: 'Select a security question...' },
    { value: 'What was the name of your first pet?', label: 'What was the name of your first pet?' },
    { value: 'What city were you born in?', label: 'What city were you born in?' },
    { value: 'What is your mother\'s maiden name?', label: 'What is your mother\'s maiden name?' },
    { value: 'What was the model of your first car?', label: 'What was the model of your first car?' },
    { value: 'What elementary school did you attend?', label: 'What elementary school did you attend?' },
  ];

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Basic Information */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Basic Information</h3>
        
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="First Name"
            value={formData.first_name}
            onChange={(e) => handleInputChange('first_name', e.target.value)}
            error={errors.first_name}
            placeholder="John"
            required
          />
          
          <Input
            label="Last Name"
            value={formData.last_name}
            onChange={(e) => handleInputChange('last_name', e.target.value)}
            error={errors.last_name}
            placeholder="Doe"
            required
          />
        </div>

        <Input
          label="Email Address"
          type="email"
          value={formData.email}
          onChange={(e) => handleInputChange('email', e.target.value)}
          error={errors.email}
          placeholder="john.doe@example.com"
          required
          helperText="Used for login and system notifications"
        />

        <div className="grid grid-cols-2 gap-4">
          <Input
            label="Username"
            value={formData.username}
            onChange={(e) => handleInputChange('username', e.target.value)}
            error={errors.username}
            placeholder="johndoe"
            helperText="Optional alternative login method"
          />
          
          <Input
            label="Phone Number"
            value={formData.phone}
            onChange={(e) => handleInputChange('phone', e.target.value)}
            placeholder="+1 (555) 123-4567"
            helperText="Optional contact information"
          />
        </div>
      </div>

      {/* Password Section */}
      <div className="border-t border-gray-200 pt-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-gray-900">
            {initialData ? 'Change Password' : 'Password'}
          </h3>
          {initialData && (
            <Checkbox
              label="Change password"
              checked={showPasswordFields}
              onChange={setShowPasswordFields}
            />
          )}
        </div>

        {showPasswordFields && (
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="Password"
              type="password"
              value={formData.password}
              onChange={(e) => handleInputChange('password', e.target.value)}
              error={errors.password}
              placeholder="••••••••"
              required={!initialData}
              helperText="Minimum 8 characters"
            />
            
            <Input
              label="Confirm Password"
              type="password"
              value={formData.confirm_password}
              onChange={(e) => handleInputChange('confirm_password', e.target.value)}
              error={errors.confirm_password}
              placeholder="••••••••"
              required={!initialData}
            />
          </div>
        )}
      </div>

      {/* Security Question */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium text-gray-900">Security Question</h3>
        
        <Select
          label="Security Question"
          value={formData.security_question}
          onChange={(value) => handleInputChange('security_question', value)}
          options={SECURITY_QUESTIONS}
          placeholder="Select a security question..."
          helperText="Used for password recovery"
        />

        {formData.security_question && (
          <Input
            label="Security Answer"
            value={formData.security_answer}
            onChange={(e) => handleInputChange('security_answer', e.target.value)}
            placeholder="Your answer..."
            helperText="Keep this answer safe - it's used for account recovery"
          />
        )}
      </div>

      {/* User Settings */}
      <div className="border-t border-gray-200 pt-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">User Settings</h3>
        
        <div className="space-y-3">
          <Checkbox
            label="Active User"
            checked={formData.is_active}
            onChange={(checked) => handleInputChange('is_active', checked)}
            helperText="Inactive users cannot log in to the system"
          />

          <Checkbox
            label="System Administrator"
            checked={formData.is_sys_admin}
            onChange={(checked) => handleInputChange('is_sys_admin', checked)}
            helperText="System administrators have full access to all features"
          />
        </div>
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
          {initialData ? 'Update User' : 'Create User'}
        </Button>
      </div>
    </form>
  );
};

export default UserForm;