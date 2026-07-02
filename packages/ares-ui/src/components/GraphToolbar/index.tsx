import React from 'react';

export const GraphToolbar = ({ children, className = '' }: { children: React.ReactNode, className?: string }) => {
    return (
        <div className={`flex items-center space-x-2 p-2 bg-gray-50 border-b ${className}`}>
            {children}
        </div>
    );
};
