import { useMemo } from 'react';
import { 
    Users, 
    Zap, 
    Activity, 
    AlertTriangle,
    ArrowRight,
    RefreshCw
} from 'lucide-react';
import { useAccountStore } from '../../stores/useAccountStore';
import { useConfigStore } from '../../stores/useConfigStore';
import { showToast } from '../common/ToastContainer';
import { formatNumber } from '../../utils/format';
import { cn } from '../../utils/cn';

interface StatCardProps {
    title: string;
    value: string | number;
    subtitle?: string;
    icon: React.ReactNode;
    color: 'blue' | 'emerald' | 'amber' | 'rose' | 'purple';
    trend?: 'up' | 'down' | 'neutral';
}

function StatCard({ title, value, subtitle, icon, color }: StatCardProps) {
    const colorClasses = {
        blue: 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400',
        emerald: 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-600 dark:text-emerald-400',
        amber: 'bg-amber-50 dark:bg-amber-900/20 text-amber-600 dark:text-amber-400',
        rose: 'bg-rose-50 dark:bg-rose-900/20 text-rose-600 dark:text-rose-400',
        purple: 'bg-purple-50 dark:bg-purple-900/20 text-purple-600 dark:text-purple-400',
    };

    return (
        <div className="bg-white dark:bg-gray-900 rounded-2xl p-5 border border-gray-200 dark:border-gray-800 hover:shadow-lg transition-shadow">
            <div className="flex items-start justify-between">
                <div className={cn('p-3 rounded-xl', colorClasses[color])}>
                    {icon}
                </div>
            </div>
            <div className="mt-4">
                <p className="text-2xl font-bold text-gray-900 dark:text-white">{value}</p>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">{title}</p>
                {subtitle && (
                    <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">{subtitle}</p>
                )}
            </div>
        </div>
    );
}

export default function StatsOverview() {
    const { accounts, currentAccount, refreshAllQuotas, loading } = useAccountStore();
    const { config } = useConfigStore();

    const stats = useMemo(() => {
        const total = accounts.length;
        const active = accounts.filter(a => a.isActive).length;
        const disabled = accounts.filter(a => a.disabled).length;
        const lowQuota = accounts.filter(a => {
            const quota = a.quota?.remainingRequests || 0;
            const total = a.quota?.totalRequests || 1;
            return (quota / total) < 0.2;
        }).length;

        const totalRequests = accounts.reduce((sum, a) => sum + (a.usage?.dailyRequests || 0), 0);
        const totalTokens = accounts.reduce((sum, a) => sum + (a.usage?.dailyTokens || 0), 0);

        const avgQuota = accounts.length > 0
            ? Math.round(
                accounts.reduce((sum, a) => {
                    const quota = a.quota?.remainingRequests || 0;
                    const total = a.quota?.totalRequests || 1;
                    return sum + (quota / total) * 100;
                }, 0) / accounts.length
            )
            : 0;

        return {
            total,
            active,
            disabled,
            lowQuota,
            totalRequests,
            totalTokens,
            avgQuota,
        };
    }, [accounts]);

    const handleRefreshAll = async () => {
        try {
            await refreshAllQuotas();
            showToast('All quotas refreshed', 'success');
        } catch (error) {
            showToast(`Failed to refresh: ${error}`, 'error');
        }
    };

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Dashboard</h2>
                    <p className="text-gray-500 dark:text-gray-400 mt-1">
                        Manage your OpenAI API accounts and monitor usage
                    </p>
                </div>
                <button
                    onClick={handleRefreshAll}
                    disabled={loading}
                    className="inline-flex items-center gap-2 px-4 py-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors disabled:opacity-50"
                >
                    <RefreshCw className={cn('w-4 h-4', loading && 'animate-spin')} />
                    Refresh All
                </button>
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <StatCard
                    title="Total Accounts"
                    value={stats.total}
                    subtitle={`${stats.active} active, ${stats.disabled} disabled`}
                    icon={<Users className="w-6 h-6" />}
                    color="blue"
                />
                <StatCard
                    title="Daily Requests"
                    value={formatNumber(stats.totalRequests)}
                    subtitle="Across all accounts"
                    icon={<Activity className="w-6 h-6" />}
                    color="emerald"
                />
                <StatCard
                    title="Average Quota"
                    value={`${stats.avgQuota}%`}
                    subtitle={`${stats.lowQuota} accounts low on quota`}
                    icon={<Zap className="w-6 h-6" />}
                    color="purple"
                />
                <StatCard
                    title="Low Quota Alerts"
                    value={stats.lowQuota}
                    subtitle="Accounts below 20%"
                    icon={<AlertTriangle className="w-6 h-6" />}
                    color={stats.lowQuota > 0 ? 'rose' : 'amber'}
                />
            </div>

            {/* Current Account Card */}
            {currentAccount && (
                <div className="bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl p-6 text-white">
                    <div className="flex items-start justify-between">
                        <div>
                            <p className="text-blue-100 text-sm font-medium">Current Active Account</p>
                            <h3 className="text-2xl font-bold mt-1">{currentAccount.name}</h3>
                            <p className="text-blue-200 text-sm mt-1">
                                {currentAccount.quota?.tier?.toUpperCase() || 'UNKNOWN'} Tier
                            </p>
                        </div>
                        <div className="bg-white/20 backdrop-blur-sm px-3 py-1.5 rounded-lg">
                            <span className="text-sm font-medium">Active</span>
                        </div>
                    </div>

                    <div className="grid grid-cols-3 gap-4 mt-6 pt-6 border-t border-white/20">
                        <div>
                            <p className="text-blue-200 text-xs uppercase">Remaining Requests</p>
                            <p className="text-xl font-semibold mt-1">
                                {formatNumber(currentAccount.quota?.remainingRequests || 0)}
                            </p>
                        </div>
                        <div>
                            <p className="text-blue-200 text-xs uppercase">Daily Usage</p>
                            <p className="text-xl font-semibold mt-1">
                                {formatNumber(currentAccount.usage?.dailyRequests || 0)}
                            </p>
                        </div>
                        <div>
                            <p className="text-blue-200 text-xs uppercase">Monthly Usage</p>
                            <p className="text-xl font-semibold mt-1">
                                {formatNumber(currentAccount.usage?.monthlyRequests || 0)}
                            </p>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
