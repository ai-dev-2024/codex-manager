import { useState } from 'react';
import { createPortal } from 'react-dom';
import { X, Copy, Check } from 'lucide-react';
import { OpenAIAccount } from '../../types/account';
import { formatDate, maskApiKey, formatNumber } from '../../utils/format';
import { cn } from '../../utils/cn';

interface AccountDetailsDialogProps {
    account: OpenAIAccount | null;
    onClose: () => void;
}

export default function AccountDetailsDialog({ account, onClose }: AccountDetailsDialogProps) {
    const [copied, setCopied] = useState(false);

    if (!account) return null;

    const copyApiKey = async () => {
        try {
            await navigator.clipboard.writeText(account.apiKey);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
        }
    };

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center">
            <div 
                className="absolute inset-0 bg-black/50 backdrop-blur-sm"
                onClick={onClose}
            />
            
            <div className="relative bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-lg mx-4 max-h-[90vh] overflow-auto">
                {/* Header */}
                <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-800">
                    <div>
                        <h2 className="text-xl font-bold text-gray-900 dark:text-white">{account.name}</h2>
                        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                            Created {formatDate(account.createdAt)}
                        </p>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
                    >
                        <X className="w-5 h-5 text-gray-500" />
                    </button>
                </div>

                <div className="p-6 space-y-6">
                    {/* Status */}
                    <div className="flex gap-3">
                        <div className={cn(
                            'px-3 py-1.5 rounded-lg text-sm font-medium',
                            account.isActive
                                ? 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-400'
                                : 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400'
                        )}>
                            {account.isActive ? 'Active' : 'Inactive'}
                        </div>
                        {account.disabled && (
                            <div className="px-3 py-1.5 rounded-lg text-sm font-medium bg-rose-100 dark:bg-rose-900/30 text-rose-700 dark:text-rose-400">
                                Disabled
                            </div>
                        )}
                        {account.quota?.tier && (
                            <div className="px-3 py-1.5 rounded-lg text-sm font-medium bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 uppercase">
                                {account.quota.tier}
                            </div>
                        )}
                    </div>

                    {/* API Key */}
                    <div className="space-y-2">
                        <label className="text-sm font-medium text-gray-700 dark:text-gray-300">API Key</label>
                        <div className="flex gap-2">
                            <div className="flex-1 px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl font-mono text-sm text-gray-600 dark:text-gray-400">
                                {maskApiKey(account.apiKey)}
                            </div>
                            <button
                                onClick={copyApiKey}
                                className="px-4 py-2.5 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-xl transition-colors"
                            >
                                {copied ? (
                                    <Check className="w-5 h-5 text-emerald-500" />
                                ) : (
                                    <Copy className="w-5 h-5 text-gray-500" />
                                )}
                            </button>
                        </div>
                    </div>

                    {/* Base URL */}
                    <div className="space-y-2">
                        <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Base URL</label>
                        <div className="px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-sm text-gray-600 dark:text-gray-400">
                            {account.baseUrl || 'https://api.openai.com/v1'}
                        </div>
                    </div>

                    {/* Organization */}
                    {account.organization && (
                        <div className="space-y-2">
                            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Organization</label>
                            <div className="px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-sm text-gray-600 dark:text-gray-400">
                                {account.organization}
                            </div>
                        </div>
                    )}

                    {/* Quota Info */}
                    <div className="pt-4 border-t border-gray-200 dark:border-gray-800">
                        <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-4">Quota Information</h3>
                        
                        <div className="grid grid-cols-2 gap-4">
                            <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Remaining Requests</p>
                                <p className="text-lg font-semibold text-gray-900 dark:text-white mt-1">
                                    {formatNumber(account.quota?.remainingRequests || 0)}
                                </p>
                            </div>
                            <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Total Requests</p>
                                <p className="text-lg font-semibold text-gray-900 dark:text-white mt-1">
                                    {formatNumber(account.quota?.totalRequests || 0)}
                                </p>
                            </div>
                            <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Remaining Tokens</p>
                                <p className="text-lg font-semibold text-gray-900 dark:text-white mt-1">
                                    {formatNumber(account.quota?.remainingTokens || 0)}
                                </p>
                            </div>
                            <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Total Tokens</p>
                                <p className="text-lg font-semibold text-gray-900 dark:text-white mt-1">
                                    {formatNumber(account.quota?.totalTokens || 0)}
                                </p>
                            </div>
                        </div>
                    </div>

                    {/* Usage Stats */}
                    <div className="pt-4 border-t border-gray-200 dark:border-gray-800">
                        <h3 className="text-sm font-semibold text-gray-900 dark:text-white mb-4">Usage Statistics</h3>
                        
                        <div className="grid grid-cols-2 gap-4">
                            <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Daily Requests</p>
                                <p className="text-lg font-semibold text-gray-900 dark:text-white mt-1">
                                    {formatNumber(account.usage?.dailyRequests || 0)}
                                </p>
                            </div>
                            <div className="p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Monthly Requests</p>
                                <p className="text-lg font-semibold text-gray-900 dark:text-white mt-1">
                                    {formatNumber(account.usage?.monthlyRequests || 0)}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
