import React from 'react';

export const Sidebar = ({ children, className = '' }: { children: React.ReactNode, className?: string }) => {
    return (
        <div className={`w-64 h-full bg-gray-100 border-r flex flex-col ${className}`}>
            {children}
        </div>
    );
};
