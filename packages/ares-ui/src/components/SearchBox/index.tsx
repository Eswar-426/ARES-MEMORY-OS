
export const SearchBox = ({ value, onChange, placeholder = 'Search...' }: { value: string, onChange: (val: string) => void, placeholder?: string }) => {
    return (
        <div className="relative w-full">
            <input 
                type="text" 
                value={value}
                onChange={(e) => onChange(e.target.value)}
                placeholder={placeholder}
                className="w-full px-4 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
        </div>
    );
};
