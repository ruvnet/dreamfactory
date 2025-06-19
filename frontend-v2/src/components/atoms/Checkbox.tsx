import React from 'react';
import { CheckIcon } from '@heroicons/react/24/outline';

export interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type' | 'onChange'> {
  label?: string;
  helperText?: string;
  error?: string;
  onChange?: (checked: boolean) => void;
}

export const Checkbox: React.FC<CheckboxProps> = ({
  label,
  helperText,
  error,
  onChange,
  checked = false,
  disabled = false,
  required = false,
  className = '',
  ...props
}) => {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange?.(e.target.checked);
  };

  return (
    <div className={`space-y-1 ${className}`}>
      <div className="flex items-start space-x-3">
        <div className="relative flex items-center">
          <input
            type="checkbox"
            checked={checked}
            onChange={handleChange}
            disabled={disabled}
            required={required}
            className="sr-only"
            {...props}
          />
          <div
            className={`
              w-4 h-4 border-2 rounded flex items-center justify-center cursor-pointer transition-colors
              ${checked
                ? error
                  ? 'bg-red-500 border-red-500'
                  : 'bg-blue-500 border-blue-500'
                : error
                  ? 'border-red-300 bg-white'
                  : 'border-gray-300 bg-white'
              }
              ${disabled
                ? 'opacity-50 cursor-not-allowed'
                : 'hover:border-blue-400'
              }
            `}
            onClick={() => !disabled && onChange?.(!checked)}
          >
            {checked && (
              <CheckIcon className="w-3 h-3 text-white" strokeWidth={3} />
            )}
          </div>
        </div>
        
        {label && (
          <label
            className={`
              text-sm cursor-pointer select-none
              ${disabled ? 'text-gray-400' : 'text-gray-700'}
            `}
            onClick={() => !disabled && onChange?.(!checked)}
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
      </div>

      {error && (
        <p className="text-sm text-red-600 ml-7">{error}</p>
      )}
      
      {helperText && !error && (
        <p className="text-sm text-gray-500 ml-7">{helperText}</p>
      )}
    </div>
  );
};

export default Checkbox;