import { useMemo } from 'react';
import { ArrowRightLeft, Crown } from 'lucide-react';
import { OpenAIAccount } from '../../types/account';
import { formatNumber, getQuotaColorClass } from '../../utils/format';
import { cn } from '../../utils/cn';

interface BestAccountCardProps {
    accounts: OpenAIAccount[];
    currentAccountId?: string;
    onSwitch: (id: string) => void;
    isLoading?: boolean;
}

export default function BestAccountCard({ 
    accounts, 
    currentAccountId, 
    onSwitch,
    isLoading 
}: BestAccountCardProps) {
    const bestAccount = useMemo(() => {
        // Find account with highest remaining quota percentage
        return accounts
            .filter(a => !a.disabled)
            .sort((a, b) => {
                const aQuota = a.quota?.remainingRequests || 0;
                const aTotal = a.quota?.totalRequests || 1;
                const bQuota = b.quota?.remainingRequests || 0;
                const bTotal = b.quota?.totalRequests || 1;
                return (bQuota / bTotal) - (aQuota / aTotal);
            })[0];
    }, [accounts]);

    if (!bestAccount) {
        return (
            <div className="bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800">
                <div className="flex items-center gap-3 mb-4">
                    <div className="p-2 bg-amber-100 dark:bg-amber-900/30 rounded-xl">
                        <Crown className="w-5 h-5 text-amber-600 dark:text-amber-400" />
                    </div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Best Account</h3>
                </div>
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                    No available accounts
                </p>
            </div>
        );
    }

    const quotaPercentage = bestAccount.quota?.remainingRequests && bestAccount.quota?.totalRequests
        ? Math.round((bestAccount.quota.remainingRequests / bestAccount.quota.totalRequests) * 100)
        : 0;

    const isCurrent = bestAccount.id === currentAccountId;

    return (
        <div className="bg-white dark:bg-gray-900 rounded-2xl p-6 border border-gray-200 dark:border-gray-800">
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-amber-100 dark:bg-amber-900/30 rounded-xl">
                        <Crown className="w-5 h-5 text-amber-600 dark:text-amber-400" />
                    </div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Best Account</h3>
                </div>
                {!isCurrent && (
                    <button
                        onClick={() => onSwitch(bestAccount.id)}
                        disabled={isLoading}
                        className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-xs font-medium rounded-lg transition-colors disabled:opacity-50"
                    >
                        <ArrowRightLeft className="w-3.5 h-3.5" />
                        Switch
                    </button>
                )}
            </div>

            <div className="space-y-4">
                <div>
                    <p className="text-sm text-gray-500 dark:text-gray-400">Account Name</p>
                    <p className="text-lg font-semibold text-gray-900 dark:text-white">{bestAccount.name}</p>
                </div>

                <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                        <span className="text-gray-500 dark:text-gray-400">Quota Remaining</span>
                        <span className="font-medium text-gray-700 dark:text-gray-300">{quotaPercentage}%</span>
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

                <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-100 dark:border-gray-800">
                    <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Requests</p>
                        <p className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                            {formatNumber(bestAccount.quota?.remainingRequests || 0)} / {formatNumber(bestAccount.quota?.totalRequests || 0)}
                        </p>
                    </div>
                    <div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 uppercase">Tier</p>
                        <p className="text-sm font-semibold text-gray-700 dark:text-gray-300 uppercase">
                            {bestAccount.quota?.tier || 'Unknown'}
                        </p>
                    </div>
                </div>

                {isCurrent && (
                    <div className="mt-4 p-3 bg-emerald-50 dark:bg-emerald-900/20 rounded-xl">
                        <p className="text-sm text-emerald-700 dark:text-emerald-400 text-center font-medium">
                            This is your current active account
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}
