/**
 * DataTable Component
 * 
 * TODO: Implement a responsive data table with:
 * - Column sorting (click headers)
 * - Text search filtering
 * - Pagination with configurable page size
 * - Responsive design (stack on mobile, scroll on tablet, full on desktop)
 * - Keyboard navigation support
 * - Loading and empty states
 */

import React from 'react';

// Placeholder - implement the full DataTable component
export interface Column<T> {
  id: string;
  header: string;
  accessor: keyof T | ((row: T) => React.ReactNode);
  sortable?: boolean;
}

export interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  pageSize?: number;
  loading?: boolean;
  onRowClick?: (row: T) => void;
}

export function DataTable<T extends { id: string | number }>({
  columns,
  data,
  pageSize = 10,
  loading = false,
  onRowClick,
}: DataTableProps<T>) {
  // TODO: Implement using @tanstack/react-table
  // TODO: Add sorting state
  // TODO: Add filtering state
  // TODO: Add pagination state
  // TODO: Add responsive breakpoints
  
  if (loading) {
    return <div className="animate-pulse">Loading...</div>;
  }

  if (data.length === 0) {
    return <div className="text-gray-500 text-center py-8">No data available</div>;
  }

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200">
        <thead className="bg-gray-50">
          <tr>
            {columns.map((column) => (
              <th
                key={column.id}
                className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="bg-white divide-y divide-gray-200">
          {data.slice(0, pageSize).map((row) => (
            <tr
              key={row.id}
              onClick={() => onRowClick?.(row)}
              className={onRowClick ? 'cursor-pointer hover:bg-gray-50' : ''}
            >
              {columns.map((column) => (
                <td key={column.id} className="px-6 py-4 whitespace-nowrap">
                  {typeof column.accessor === 'function'
                    ? column.accessor(row)
                    : String(row[column.accessor])}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {/* TODO: Add pagination controls */}
    </div>
  );
}

export default DataTable;



