import { 
    ArrowRightLeft, 
    RefreshCw, 
    Trash2, 
    Download, 
    Info, 
    Ban, 
    Lock,
    Clock,
    ToggleLeft,
    ToggleRight,
    Zap,
    Layers
} from 'lucide-react';
import { OpenAIAccount } from '../../types/account';
import { getQuotaColorClass, formatTimeRemaining, formatNumber, maskApiKey } from '../../utils/format';
import { cn } from '../../utils/cn';

interface AccountCardProps {
    account: OpenAIAccount;
    selected: boolean;
    onSelect: () => void;
    isCurrent: boolean;
    isRefreshing: boolean;
    isSwitching?: boolean;
    onSwitch: () => void;
    onRefresh: () => void;
    onViewDetails: () => void;
    onExport: () => void;
    onDelete: () => void;
    onToggleStatus: () => void;
}

export default function AccountCard({
    account,
    selected,
    onSelect,
    isCurrent,
    isRefreshing,
    isSwitching = false,
    onSwitch,
    onRefresh,
    onViewDetails,
    onExport,
    onDelete,
    onToggleStatus,
}: AccountCardProps) {
    const quotaPercentage = account.quota?.remainingRequests && account.quota?.totalRequests
        ? Math.round((account.quota.remainingRequests / account.quota.totalRequests) * 100)
        : 0;

    const tokenPercentage = account.quota?.remainingTokens && account.quota?.totalTokens
        ? Math.round((account.quota.remainingTokens / account.quota.totalTokens) * 100)
        : 0;

    const tierColors: Record<string, string> = {
        free: 'bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400',
        tier1: 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400',
        tier2: 'bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400',
        tier3: 'bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400',
        tier4: 'bg-pink-100 dark:bg-pink-900/30 text-pink-600 dark:text-pink-400',
        tier5: 'bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400',
    };

    return (
        <div
            className={cn(
                'flex flex-col p-4 rounded-2xl border transition-all hover:shadow-lg',
                isCurrent
                    ? 'bg-blue-50/50 dark:bg-blue-900/10 border-blue-200 dark:border-blue-800'
                    : 'bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800',
                (isRefreshing || account.disabled) && 'opacity-70'
            )}
        >
            {/* Header */}
            <div className="flex items-start gap-3 mb-3">
                <input
                    type="checkbox"
                    checked={selected}
                    onChange={onSelect}
                    onClick={(e) => e.stopPropagation()}
                    className="mt-1 w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                        <h3
                            className={cn(
                                'font-semibold text-sm truncate',
                                isCurrent
                                    ? 'text-blue-700 dark:text-blue-400'
                                    : 'text-gray-900 dark:text-white'
                            )}
                            title={account.name}
                        >
                            {account.name}
                        </h3>
                        <div className="flex items-center gap-1">
                            {isCurrent && (
                                <span className="px-2 py-0.5 rounded-md bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 text-[10px] font-bold">
                                    ACTIVE
                                </span>
                            )}
                            {account.disabled && (
                                <span className="px-2 py-0.5 rounded-md bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300 text-[10px] font-bold flex items-center gap-1">
                                    <Ban className="w-3 h-3" />
                                    DISABLED
                                </span>
                            )}
                            {account.quota?.tier && (
                                <span className={cn(
                                    'px-2 py-0.5 rounded-md text-[10px] font-bold uppercase',
                                    tierColors[account.quota.tier]
                                )}>
                                    {account.quota.tier}
                                </span>
                            )}
                        </div>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                        {maskApiKey(account.apiKey)}
                    </p>
                </div>
            </div>

            {/* Quota Section */}
            <div className="space-y-3 mb-4">
                {/* Requests Quota */}
                <div className="space-y-1">
                    <div className="flex justify-between text-xs">
                        <span className="text-gray-500 dark:text-gray-400">Requests</span>
                        <span className="font-medium text-gray-700 dark:text-gray-300">
                            {formatNumber(account.quota?.remainingRequests || 0)} / {formatNumber(account.quota?.totalRequests || 0)}
                        </span>
                    </div>
                    <div className="h-2 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden">
                        <div
                            className={cn(
                                'h-full rounded-full transition-all duration-500',
                                getQuotaColorClass(quotaPercentage).split(' ')[0]
                            )}
                            style={{ width: `${quotaPercentage}%` }}
                        />
                    </div>
                </div>

                {/* Tokens Quota */}
                <div className="space-y-1">
                    <div className="flex justify-between text-xs">
                        <span className="text-gray-500 dark:text-gray-400">Tokens</span>
                        <span className="font-medium text-gray-700 dark:text-gray-300">
                            {formatNumber(account.quota?.remainingTokens || 0)} / {formatNumber(account.quota?.totalTokens || 0)}
                        </span>
                    </div>
                    <div className="h-2 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden">
                        <div
                            className={cn(
                                'h-full rounded-full transition-all duration-500',
                                getQuotaColorClass(tokenPercentage).split(' ')[0]
                            )}
                            style={{ width: `${tokenPercentage}%` }}
                        />
                    </div>
                </div>

                {/* Reset Time */}
                {account.quota?.resetTime && (
                    <div className="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400">
                        <Clock className="w-3.5 h-3.5" />
                        Resets in {formatTimeRemaining(account.quota.resetTime)}
                    </div>
                )}
            </div>

            {/* Usage Stats */}
            <div className="grid grid-cols-2 gap-2 mb-4 p-3 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
                <div>
                    <p className="text-[10px] text-gray-500 dark:text-gray-400 uppercase">Daily</p>
                    <p className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                        {formatNumber(account.usage?.dailyRequests || 0)} req
                    </p>
                </div>
                <div>
                    <p className="text-[10px] text-gray-500 dark:text-gray-400 uppercase">Monthly</p>
                    <p className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                        {formatNumber(account.usage?.monthlyRequests || 0)} req
                    </p>
                </div>
            </div>

            {/* Actions */}
            <div className="flex items-center justify-between pt-3 border-t border-gray-100 dark:border-gray-800">
                <span className="text-xs text-gray-400 dark:text-gray-500">
                    {account.lastUsed ? `Last used ${formatTimeRemaining(new Date(account.lastUsed * 1000).toISOString())}` : 'Never used'}
                </span>

                <div className="flex items-center gap-1">
                    <button
                        onClick={(e) => { e.stopPropagation(); onViewDetails(); }}
                        className="p-2 text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition-all"
                        title="View Details"
                    >
                        <Info className="w-4 h-4" />
                    </button>
                    <button
                        onClick={(e) => { e.stopPropagation(); onSwitch(); }}
                        disabled={isSwitching || account.disabled}
                        className={cn(
                            'p-2 rounded-lg transition-all',
                            (isSwitching || account.disabled)
                                ? 'text-gray-300 cursor-not-allowed'
                                : 'text-gray-400 hover:text-emerald-600 dark:hover:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-900/20'
                        )}
                        title={account.disabled ? 'Account disabled' : 'Switch to this account'}
                    >
                        <ArrowRightLeft className={cn('w-4 h-4', isSwitching && 'animate-spin')} />
                    </button>
                    <button
                        onClick={(e) => { e.stopPropagation(); onRefresh(); }}
                        disabled={isRefreshing}
                        className={cn(
                            'p-2 rounded-lg transition-all',
                            isRefreshing
                                ? 'text-gray-300'
                                : 'text-gray-400 hover:text-blue-600 hover:bg-blue-50'
                        )}
                        title="Refresh quota"
                    >
                        <RefreshCw className={cn('w-4 h-4', isRefreshing && 'animate-spin')} />
                    </button>
                    <button
                        onClick={(e) => { e.stopPropagation(); onExport(); }}
                        className="p-2 text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 rounded-lg transition-all"
                        title="Export"
                    >
                        <Download className="w-4 h-4" />
                    </button>
                    <button
                        onClick={(e) => { e.stopPropagation(); onToggleStatus(); }}
                        className={cn(
                            'p-2 rounded-lg transition-all',
                            account.disabled
                                ? 'text-gray-400 hover:text-emerald-600 hover:bg-emerald-50'
                                : 'text-gray-400 hover:text-amber-600 hover:bg-amber-50'
                        )}
                        title={account.disabled ? 'Enable account' : 'Disable account'}
                    >
                        {account.disabled ? (
                            <ToggleRight className="w-4 h-4" />
                        ) : (
                            <ToggleLeft className="w-4 h-4" />
                        )}
                    </button>
                    <button
                        onClick={(e) => { e.stopPropagation(); onDelete(); }}
                        className="p-2 text-gray-400 hover:text-rose-600 hover:bg-rose-50 rounded-lg transition-all"
                        title="Delete"
                    >
                        <Trash2 className="w-4 h-4" />
                    </button>
                </div>
            </div>
        </div>
    );
}
