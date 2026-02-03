import { useState, useEffect, useCallback } from 'react';
import { X, CheckCircle, AlertCircle, Info } from 'lucide-react';
import { cn } from '../../utils/cn';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface ToastItem {
    id: string;
    message: string;
    type: ToastType;
    duration?: number;
}

interface ToastProps extends ToastItem {
    onClose: (id: string) => void;
}

function Toast({ id, message, type, duration = 3000, onClose }: ToastProps) {
    const [isExiting, setIsExiting] = useState(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            setIsExiting(true);
            setTimeout(() => onClose(id), 300);
        }, duration);

        return () => clearTimeout(timer);
    }, [id, duration, onClose]);

    const icons = {
        success: <CheckCircle className="w-5 h-5 text-emerald-500" />,
        error: <AlertCircle className="w-5 h-5 text-rose-500" />,
        warning: <AlertCircle className="w-5 h-5 text-amber-500" />,
        info: <Info className="w-5 h-5 text-blue-500" />,
    };

    const bgColors = {
        success: 'bg-emerald-50 dark:bg-emerald-900/20 border-emerald-200 dark:border-emerald-800',
        error: 'bg-rose-50 dark:bg-rose-900/20 border-rose-200 dark:border-rose-800',
        warning: 'bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800',
        info: 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800',
    };

    return (
        <div
            className={cn(
                'flex items-center gap-3 px-4 py-3 rounded-xl shadow-lg border min-w-[300px] max-w-md',
                'transform transition-all duration-300',
                isExiting ? 'translate-x-full opacity-0' : 'translate-x-0 opacity-100',
                bgColors[type]
            )}
        >
            {icons[type]}
            <span className="flex-1 text-sm font-medium text-gray-900 dark:text-gray-100">
                {message}
            </span>
            <button
                onClick={() => {
                    setIsExiting(true);
                    setTimeout(() => onClose(id), 300);
                }}
                className="p-1 hover:bg-black/5 dark:hover:bg-white/10 rounded-lg transition-colors"
            >
                <X className="w-4 h-4 text-gray-500" />
            </button>
        </div>
    );
}

let toastCounter = 0;
let addToastExternal: ((message: string, type: ToastType, duration?: number) => void) | null = null;

export const showToast = (message: string, type: ToastType = 'info', duration: number = 3000) => {
    if (addToastExternal) {
        addToastExternal(message, type, duration);
    } else {
        console.warn('ToastContainer not mounted');
    }
};

export default function ToastContainer() {
    const [toasts, setToasts] = useState<ToastItem[]>([]);

    const addToast = useCallback((message: string, type: ToastType, duration?: number) => {
        const id = `toast-${Date.now()}-${toastCounter++}`;
        setToasts((prev) => [...prev, { id, message, type, duration }]);
    }, []);

    const removeToast = useCallback((id: string) => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
    }, []);

    useEffect(() => {
        addToastExternal = addToast;
        return () => {
            addToastExternal = null;
        };
    }, [addToast]);

    return (
        <div className="fixed top-24 right-6 z-[200] flex flex-col gap-3 pointer-events-none">
            <div className="flex flex-col gap-3 pointer-events-auto">
                {toasts.map((toast) => (
                    <Toast key={toast.id} {...toast} onClose={removeToast} />
                ))}
            </div>
        </div>
    );
}
