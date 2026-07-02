import React from 'react';
import { Card } from '../Card';

export const MetricCard = ({ title, value, icon }: { title: string, value: string | number, icon?: React.ReactNode }) => {
    return (
        <Card className="flex flex-col">
            <div className="flex items-center justify-between">
                <span className="text-gray-500 text-sm font-medium">{title}</span>
                {icon && <span className="text-gray-400">{icon}</span>}
            </div>
            <span className="text-2xl font-bold mt-2">{value}</span>
        </Card>
    );
};
