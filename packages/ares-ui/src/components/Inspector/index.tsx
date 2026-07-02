import React from 'react';

export const Inspector = ({ title, children, isOpen, onClose }: { title: string, children: React.ReactNode, isOpen: boolean, onClose: () => void }) => {
    if (!isOpen) return null;
    return (
        <div className="absolute right-0 top-0 h-full w-80 bg-white shadow-xl border-l flex flex-col z-50 transition-transform">
            <div className="p-4 border-b flex justify-between items-center">
                <h3 className="font-semibold text-lg">{title}</h3>
                <button onClick={onClose} className="text-gray-500 hover:text-gray-700">&times;</button>
            </div>
            <div className="p-4 flex-grow overflow-y-auto">
                {children}
            </div>
        </div>
    );
};
