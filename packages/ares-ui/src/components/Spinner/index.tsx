import React from 'react';

export const Spinner = ({ size = 'md' }: { size?: 'sm' | 'md' | 'lg' }) => {
    const sizeClasses = size === 'sm' ? 'w-4 h-4' : size === 'lg' ? 'w-8 h-8' : 'w-6 h-6';
    return (
        <div className={`animate-spin rounded-full border-b-2 border-gray-900 ${sizeClasses}`}></div>
    );
};
