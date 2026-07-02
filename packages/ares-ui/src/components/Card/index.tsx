import React from 'react';

export const Card = ({ children, className = '' }: { children: React.ReactNode, className?: string }) => {
    return (
        <div className={`bg-white shadow rounded-lg p-4 ${className}`}>
            {children}
        </div>
    );
};
