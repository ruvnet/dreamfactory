import React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';

const cardVariants = cva(
  'bg-white rounded-lg border border-gray-200 shadow-sm',
  {
    variants: {
      variant: {
        default: 'border-gray-200',
        elevated: 'shadow-md border-gray-300',
        outlined: 'border-2 border-gray-300 shadow-none',
      },
      padding: {
        none: '',
        sm: 'p-4',
        md: 'p-6',
        lg: 'p-8',
      },
    },
    defaultVariants: {
      variant: 'default',
      padding: 'md',
    },
  }
);

export interface CardProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof cardVariants> {}

export const Card: React.FC<CardProps> = ({
  className,
  variant,
  padding,
  children,
  ...props
}) => {
  return (
    <div className={cardVariants({ variant, padding, className })} {...props}>
      {children}
    </div>
  );
};

export default Card;