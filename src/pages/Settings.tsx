import { useState, useEffect } from 'react';
import { Save, Globe, Bell, Palette, Server, Shield } from 'lucide-react';
import { useConfigStore } from '../stores/useConfigStore';
import { showToast } from '../components/common/ToastContainer';
import { cn } from '../utils/cn';

type TabType = 'general' | 'proxy' | 'notifications' | 'appearance';

export default function Settings() {
    const { config, loadConfig, saveConfig, updateTheme, updateLanguage, updateProxy, updateNotifications, updateAppearance } = useConfigStore();
    const [activeTab, setActiveTab] = useState<TabType>('general');
    const [isSaving, setIsSaving] = useState(false);

    useEffect(() => {
        loadConfig();
    }, []);

    const handleSave = async () => {
        setIsSaving(true);
        try {
            await saveConfig(config);
            showToast('Settings saved successfully', 'success');
        } catch (error) {
            showToast(`Failed to save: ${error}`, 'error');
        } finally {
            setIsSaving(false);
        }
    };

    const tabs = [
        { id: 'general', label: 'General', icon: Globe },
        { id: 'proxy', label: 'Proxy', icon: Server },
        { id: 'notifications', label: 'Notifications', icon: Bell },
        { id: 'appearance', label: 'Appearance', icon: Palette },
    ];

    return (
        <div className="max-w-4xl mx-auto">
            <div className="flex items-center justify-between mb-6">
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Settings</h1>
                <button
                    onClick={handleSave}
                    disabled={isSaving}
                    className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-xl transition-colors disabled:opacity-50"
                >
                    <Save className="w-4 h-4" />
                    {isSaving ? 'Saving...' : 'Save Changes'}
                </button>
            </div>

            <div className="flex gap-6">
                {/* Sidebar */}
                <div className="w-48 flex-shrink-0">
                    <nav className="space-y-1">
                        {tabs.map((tab) => {
                            const Icon = tab.icon;
                            return (
                                <button
                                    key={tab.id}
                                    onClick={() => setActiveTab(tab.id as TabType)}
                                    className={cn(
                                        'w-full flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all',
                                        activeTab === tab.id
                                            ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400'
                                            : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800'
                                    )}
                                >
                                    <Icon className="w-5 h-5" />
                                    {tab.label}
                                </button>
                            );
                        })}
                    </nav>
                </div>

                {/* Content */}
                <div className="flex-1 bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 p-6">
                    {activeTab === 'general' && (
                        <div className="space-y-6">
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">General Settings</h2>

                            <div className="space-y-4">
                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        Language
                                    </label>
                                    <select
                                        value={config.language}
                                        onChange={(e) => updateLanguage(e.target.value)}
                                        className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    >
                                        <option value="en">English</option>
                                        <option value="zh">简体中文</option>
                                        <option value="zh-TW">繁體中文</option>
                                        <option value="ja">日本語</option>
                                        <option value="ko">한국어</option>
                                        <option value="es">Español</option>
                                        <option value="fr">Français</option>
                                        <option value="de">Deutsch</option>
                                    </select>
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        Theme
                                    </label>
                                    <select
                                        value={config.theme}
                                        onChange={(e) => updateTheme(e.target.value as 'light' | 'dark' | 'system')}
                                        className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    >
                                        <option value="light">Light</option>
                                        <option value="dark">Dark</option>
                                        <option value="system">System</option>
                                    </select>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Auto-switch Account</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Automatically switch to best account when quota is low</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.autoSwitch}
                                            onChange={(e) => {
                                                const newConfig = { ...config, autoSwitch: e.target.checked };
                                                saveConfig(newConfig);
                                            }}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Auto-refresh Quotas</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Automatically refresh account quotas</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.autoRefresh}
                                            onChange={(e) => {
                                                const newConfig = { ...config, autoRefresh: e.target.checked };
                                                saveConfig(newConfig);
                                            }}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                {config.autoRefresh && (
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                            Refresh Interval (minutes)
                                        </label>
                                        <input
                                            type="number"
                                            min="1"
                                            max="60"
                                            value={config.refreshInterval}
                                            onChange={(e) => {
                                                const newConfig = { ...config, refreshInterval: parseInt(e.target.value) || 5 };
                                                saveConfig(newConfig);
                                            }}
                                            className="w-32 px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    {activeTab === 'proxy' && (
                        <div className="space-y-6">
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Proxy Configuration</h2>

                            <div className="space-y-4">
                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Enable Proxy</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Start the OpenAI API proxy server</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.proxy.enabled}
                                            onChange={(e) => updateProxy({ enabled: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Host</label>
                                        <input
                                            type="text"
                                            value={config.proxy.host}
                                            onChange={(e) => updateProxy({ host: e.target.value })}
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Port</label>
                                        <input
                                            type="number"
                                            value={config.proxy.port}
                                            onChange={(e) => updateProxy({ port: parseInt(e.target.value) || 8080 })}
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                    </div>
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">API Key</label>
                                    <input
                                        type="password"
                                        value={config.proxy.apiKey}
                                        onChange={(e) => updateProxy({ apiKey: e.target.value })}
                                        placeholder="Optional custom API key for proxy"
                                        className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Scheduling Mode</label>
                                    <select
                                        value={config.proxy.scheduling.mode}
                                        onChange={(e) => updateProxy({ 
                                            scheduling: { ...config.proxy.scheduling, mode: e.target.value as any }
                                        })}
                                        className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    >
                                        <option value="round-robin">Round Robin</option>
                                        <option value="least-used">Least Used</option>
                                        <option value="priority">Priority</option>
                                        <option value="random">Random</option>
                                        <option value="weighted">Weighted</option>
                                    </select>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Rate Limiting</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Enable request rate limiting</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.proxy.rateLimiting.enabled}
                                            onChange={(e) => updateProxy({ 
                                                rateLimiting: { ...config.proxy.rateLimiting, enabled: e.target.checked }
                                            })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>
                            </div>
                        </div>
                    )}

                    {activeTab === 'notifications' && (
                        <div className="space-y-6">
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Notification Settings</h2>

                            <div className="space-y-4">
                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Enable Notifications</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Show toast notifications</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.notifications.enabled}
                                            onChange={(e) => updateNotifications({ enabled: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Quota Alerts</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Notify when account quota is low</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.notifications.quotaAlerts}
                                            onChange={(e) => updateNotifications({ quotaAlerts: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Error Alerts</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Notify on errors</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.notifications.errorAlerts}
                                            onChange={(e) => updateNotifications({ errorAlerts: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>
                            </div>
                        </div>
                    )}

                    {activeTab === 'appearance' && (
                        <div className="space-y-6">
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Appearance</h2>

                            <div className="space-y-4">
                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Compact Mode</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Use compact spacing for dense layouts</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.appearance.compactMode}
                                            onChange={(e) => updateAppearance({ compactMode: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Show Quota Bars</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Display quota progress bars on cards</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.appearance.showQuotaBars}
                                            onChange={(e) => updateAppearance({ showQuotaBars: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-xl">
                                    <div>
                                        <p className="font-medium text-gray-900 dark:text-white">Animate Charts</p>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">Enable chart animations</p>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={config.appearance.animateCharts}
                                            onChange={(e) => updateAppearance({ animateCharts: e.target.checked })}
                                            className="sr-only peer"
                                        />
                                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
