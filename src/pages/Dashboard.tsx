import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Users, Zap, Activity, AlertTriangle, ArrowRight, Download, RefreshCw, Eye, EyeOff } from 'lucide-react';
import { useAccountStore } from '../stores/useAccountStore';
import { useConfigStore } from '../stores/useConfigStore';
import StatsOverview from '../components/dashboard/StatsOverview';
import BestAccountCard from '../components/dashboard/BestAccountCard';
import AddAccountDialog from '../components/accounts/AddAccountDialog';
import { showToast } from '../components/common/ToastContainer';
import { formatNumber } from '../utils/format';
import { cn } from '../utils/cn';

export default function Dashboard() {
    const navigate = useNavigate();
    const {
        accounts,
        currentAccount,
        fetchAccounts,
        fetchCurrentAccount,
        switchAccount,
        addAccount,
        refreshQuota,
        loading,
    } = useAccountStore();

    const { config } = useConfigStore();
    const [hideDetails, setHideDetails] = useState(false);
    const [isRefreshing, setIsRefreshing] = useState(false);

    useEffect(() => {
        fetchAccounts();
        fetchCurrentAccount();
    }, []);

    // Auto-refresh quotas every 5 minutes
    useEffect(() => {
        if (!config.autoRefresh) return;
        
        const interval = setInterval(() => {
            fetchAccounts();
        }, config.refreshInterval * 60 * 1000);

        return () => clearInterval(interval);
    }, [config.autoRefresh, config.refreshInterval, fetchAccounts]);

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

    const handleSwitch = async (accountId: string) => {
        try {
            await switchAccount(accountId);
            showToast('Account switched successfully', 'success');
        } catch (error) {
            showToast(`Failed to switch: ${error}`, 'error');
        }
    };

    const handleAddAccount = async (account) => {
        await addAccount(account);
        await fetchAccounts();
    };

    const handleRefreshCurrent = async () => {
        if (!currentAccount) return;
        setIsRefreshing(true);
        try {
            await refreshQuota(currentAccount.id);
            showToast('Quota refreshed', 'success');
        } catch (error) {
            showToast(`Failed to refresh: ${error}`, 'error');
        } finally {
            setIsRefreshing(false);
        }
    };

    const handleExport = async () => {
        try {
            const data = await useAccountStore.getState().exportAccounts();
            const blob = new Blob([data], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `codex-accounts-${new Date().toISOString().split('T')[0]}.json`;
            a.click();
            URL.revokeObjectURL(url);
            showToast('Accounts exported', 'success');
        } catch (error) {
            showToast(`Export failed: ${error}`, 'error');
        }
    };

    return (
        <div className="space-y-6 max-w-7xl mx-auto">
            {/* Header Actions */}
            <div className="flex justify-between items-center">
                <button
                    onClick={() => setHideDetails(!hideDetails)}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
                >
                    {hideDetails ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                    {hideDetails ? 'Show Details' : 'Hide Details'}
                </button>
                
                <div className="flex gap-2">
                    <AddAccountDialog onAdd={handleAddAccount} />
                    <button
                        onClick={handleRefreshCurrent}
                        disabled={isRefreshing || !currentAccount}
                        className={cn(
                            'inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all',
                            isRefreshing || !currentAccount
                                ? 'bg-gray-200 dark:bg-gray-800 text-gray-400 cursor-not-allowed'
                                : 'bg-blue-600 text-white hover:bg-blue-700'
                        )}
                    >
                        <RefreshCw className={cn('w-3.5 h-3.5', isRefreshing && 'animate-spin')} />
                        {isRefreshing ? 'Refreshing...' : 'Refresh Quota'}
                    </button>
                </div>
            </div>

            {/* Stats Cards */}
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                <div className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-2 bg-blue-50 dark:bg-blue-900/20 rounded-xl">
                            <Users className="w-4 h-4 text-blue-600 dark:text-blue-400" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats.total}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">Total Accounts</div>
                </div>

                <div className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-2 bg-emerald-50 dark:bg-emerald-900/20 rounded-xl">
                            <Zap className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats.avgQuota}%</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">Avg Quota</div>
                </div>

                <div className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-2 bg-purple-50 dark:bg-purple-900/20 rounded-xl">
                            <Activity className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-white">{formatNumber(stats.totalRequests)}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">Daily Requests</div>
                </div>

                <div className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-2 bg-amber-50 dark:bg-amber-900/20 rounded-xl">
                            <AlertTriangle className="w-4 h-4 text-amber-600 dark:text-amber-400" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats.lowQuota}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">Low Quota</div>
                </div>

                <div className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="p-2 bg-rose-50 dark:bg-rose-900/20 rounded-xl">
                            <AlertTriangle className="w-4 h-4 text-rose-600 dark:text-rose-400" />
                        </div>
                    </div>
                    <div className="text-2xl font-bold text-gray-900 dark:text-white">{stats.disabled}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">Disabled</div>
                </div>
            </div>

            {/* Two-column layout */}
            {!hideDetails && (
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <StatsOverview />
                    <BestAccountCard
                        accounts={accounts}
                        currentAccountId={currentAccount?.id}
                        onSwitch={handleSwitch}
                        isLoading={loading}
                    />
                </div>
            )}

            {/* Quick Links */}
            <div className="grid grid-cols-2 gap-4">
                <button
                    onClick={() => navigate('/accounts')}
                    className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800 hover:border-blue-300 dark:hover:border-blue-700 transition-all flex items-center justify-between group"
                >
                    <span className="text-gray-900 dark:text-white font-medium">View All Accounts</span>
                    <ArrowRight className="w-5 h-5 text-gray-400 group-hover:text-blue-500 transition-colors" />
                </button>
                <button
                    onClick={handleExport}
                    className="bg-white dark:bg-gray-900 rounded-2xl p-4 border border-gray-200 dark:border-gray-800 hover:border-blue-300 dark:hover:border-blue-700 transition-all flex items-center justify-between group"
                >
                    <span className="text-gray-900 dark:text-white font-medium">Export Data</span>
                    <Download className="w-5 h-5 text-gray-400 group-hover:text-blue-500 transition-colors" />
                </button>
            </div>
        </div>
    );
}
