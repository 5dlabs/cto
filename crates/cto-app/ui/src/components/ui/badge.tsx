import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const badgeVariants = cva(
  'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-zinc-700 focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-emerald-500/20 text-emerald-400',
        secondary: 'border-transparent bg-zinc-700 text-zinc-300',
        destructive: 'border-transparent bg-red-500/20 text-red-400',
        outline: 'border-zinc-600 text-zinc-300',
        running: 'border-transparent bg-amber-500/20 text-amber-400',
        stopped: 'border-transparent bg-zinc-600 text-zinc-300',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
