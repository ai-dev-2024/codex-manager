import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { AlertTriangle, CheckCircle, XCircle, Info, X } from 'lucide-react';
import { cn } from '../../utils/cn';

export type ModalType = 'confirm' | 'success' | 'error' | 'info';

interface ModalDialogProps {
    isOpen: boolean;
    title: string;
    message: string;
    type?: ModalType;
    onConfirm: () => void;
    onCancel?: () => void;
    confirmText?: string;
    cancelText?: string;
    isDestructive?: boolean;
}

export default function ModalDialog({
    isOpen,
    title,
    message,
    type = 'confirm',
    onConfirm,
    onCancel,
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    isDestructive = false,
}: ModalDialogProps) {
    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        if (isOpen) {
            setTimeout(() => setIsVisible(true), 10);
        } else {
            setIsVisible(false);
        }
    }, [isOpen]);

    if (!isOpen) return null;

    const icons = {
        success: <CheckCircle className="w-8 h-8 text-emerald-500" />,
        error: <XCircle className="w-8 h-8 text-rose-500" />,
        info: <Info className="w-8 h-8 text-blue-500" />,
        confirm: isDestructive ? (
            <AlertTriangle className="w-8 h-8 text-rose-500" />
        ) : (
            <AlertTriangle className="w-8 h-8 text-blue-500" />
        ),
    };

    const bgColors = {
        success: 'bg-emerald-50 dark:bg-emerald-900/20',
        error: 'bg-rose-50 dark:bg-rose-900/20',
        info: 'bg-blue-50 dark:bg-blue-900/20',
        confirm: isDestructive
            ? 'bg-rose-50 dark:bg-rose-900/20'
            : 'bg-blue-50 dark:bg-blue-900/20',
    };

    const buttonColors = {
        success: 'bg-emerald-500 hover:bg-emerald-600',
        error: 'bg-rose-500 hover:bg-rose-600',
        info: 'bg-blue-500 hover:bg-blue-600',
        confirm: isDestructive
            ? 'bg-rose-500 hover:bg-rose-600'
            : 'bg-blue-500 hover:bg-blue-600',
    };

    return createPortal(
        <div className="fixed inset-0 z-[100] flex items-center justify-center">
            {/* Backdrop */}
            <div
                className={cn(
                    'absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity duration-300',
                    isVisible ? 'opacity-100' : 'opacity-0'
                )}
                onClick={onCancel}
            />

            {/* Modal */}
            <div
                className={cn(
                    'relative bg-white dark:bg-gray-900 rounded-2xl shadow-2xl p-6 w-full max-w-md mx-4',
                    'transform transition-all duration-300',
                    isVisible ? 'scale-100 opacity-100' : 'scale-95 opacity-0'
                )}
            >
                {/* Close button */}
                {onCancel && (
                    <button
                        onClick={onCancel}
                        className="absolute top-4 right-4 p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
                    >
                        <X className="w-5 h-5 text-gray-500" />
                    </button>
                )}

                {/* Icon */}
                <div className="flex flex-col items-center text-center">
                    <div
                        className={cn(
                            'w-16 h-16 rounded-full flex items-center justify-center mb-4',
                            bgColors[type]
                        )}
                    >
                        {icons[type]}
                    </div>

                    <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2">
                        {title}
                    </h3>
                    <p className="text-gray-600 dark:text-gray-400 mb-6">{message}</p>

                    <div className="flex gap-3 w-full">
                        {onCancel && (
                            <button
                                onClick={onCancel}
                                className="flex-1 px-4 py-2.5 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 font-medium rounded-xl hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                            >
                                {cancelText}
                            </button>
                        )}
                        <button
                            onClick={onConfirm}
                            className={cn(
                                'flex-1 px-4 py-2.5 text-white font-medium rounded-xl transition-colors',
                                buttonColors[type]
                            )}
                        >
                            {confirmText}
                        </button>
                    </div>
                </div>
            </div>
        </div>,
        document.body
    );
}
