import React, { useEffect } from 'react';
import { XMarkIcon } from '@heroicons/react/24/outline';
import { CheckCircleIcon, ExclamationTriangleIcon, InformationCircleIcon, XCircleIcon } from '@heroicons/react/24/solid';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastProps {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
  onClose: (id: string) => void;
}

export const Toast: React.FC<ToastProps> = ({
  id,
  type,
  title,
  message,
  duration = 5000,
  onClose,
}) => {
  useEffect(() => {
    if (duration > 0) {
      const timer = setTimeout(() => {
        onClose(id);
      }, duration);

      return () => clearTimeout(timer);
    }
  }, [id, duration, onClose]);

  const getToastStyles = () => {
    switch (type) {
      case 'success':
        return {
          container: 'bg-green-50 border border-green-200',
          icon: <CheckCircleIcon className="h-5 w-5 text-green-400" />,
          title: 'text-green-800',
          message: 'text-green-700',
        };
      case 'error':
        return {
          container: 'bg-red-50 border border-red-200',
          icon: <XCircleIcon className="h-5 w-5 text-red-400" />,
          title: 'text-red-800',
          message: 'text-red-700',
        };
      case 'warning':
        return {
          container: 'bg-yellow-50 border border-yellow-200',
          icon: <ExclamationTriangleIcon className="h-5 w-5 text-yellow-400" />,
          title: 'text-yellow-800',
          message: 'text-yellow-700',
        };
      case 'info':
      default:
        return {
          container: 'bg-blue-50 border border-blue-200',
          icon: <InformationCircleIcon className="h-5 w-5 text-blue-400" />,
          title: 'text-blue-800',
          message: 'text-blue-700',
        };
    }
  };

  const styles = getToastStyles();

  return (
    <div className={`rounded-md p-4 ${styles.container} shadow-sm`}>
      <div className="flex">
        <div className="flex-shrink-0">
          {styles.icon}
        </div>
        <div className="ml-3 flex-1">
          <h3 className={`text-sm font-medium ${styles.title}`}>
            {title}
          </h3>
          {message && (
            <p className={`mt-1 text-sm ${styles.message}`}>
              {message}
            </p>
          )}
        </div>
        <div className="ml-4 flex-shrink-0">
          <button
            type="button"
            className={`
              rounded-md inline-flex focus:outline-none focus:ring-2 focus:ring-offset-2
              ${type === 'success' ? 'text-green-400 hover:text-green-500 focus:ring-green-500' : ''}
              ${type === 'error' ? 'text-red-400 hover:text-red-500 focus:ring-red-500' : ''}
              ${type === 'warning' ? 'text-yellow-400 hover:text-yellow-500 focus:ring-yellow-500' : ''}
              ${type === 'info' ? 'text-blue-400 hover:text-blue-500 focus:ring-blue-500' : ''}
            `}
            onClick={() => onClose(id)}
          >
            <span className="sr-only">Close</span>
            <XMarkIcon className="h-5 w-5" />
          </button>
        </div>
      </div>
    </div>
  );
};

export default Toast;