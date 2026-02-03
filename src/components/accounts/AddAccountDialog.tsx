import { useState } from 'react';
import { createPortal } from 'react-dom';
import { Plus, Key, Upload, X, Loader2, CheckCircle2, AlertCircle } from 'lucide-react';
import { OpenAIAccount } from '../../types/account';
import { cn } from '../../utils/cn';
import { showToast } from '../common/ToastContainer';

interface AddAccountDialogProps {
    onAdd: (account: Omit<OpenAIAccount, 'id' | 'createdAt'>) => Promise<void>;
}

type TabType = 'manual' | 'import';
type StatusType = 'idle' | 'loading' | 'success' | 'error';

export default function AddAccountDialog({ onAdd }: AddAccountDialogProps) {
    const [isOpen, setIsOpen] = useState(false);
    const [activeTab, setActiveTab] = useState<TabType>('manual');
    const [status, setStatus] = useState<StatusType>('idle');
    const [message, setMessage] = useState('');

    // Form fields
    const [name, setName] = useState('');
    const [apiKey, setApiKey] = useState('');
    const [baseUrl, setBaseUrl] = useState('https://api.openai.com/v1');
    const [organization, setOrganization] = useState('');
    const [importData, setImportData] = useState('');

    const resetForm = () => {
        setName('');
        setApiKey('');
        setBaseUrl('https://api.openai.com/v1');
        setOrganization('');
        setImportData('');
        setStatus('idle');
        setMessage('');
    };

    const handleClose = () => {
        setIsOpen(false);
        setTimeout(resetForm, 300);
    };

    const handleSubmit = async () => {
        if (!name.trim() || !apiKey.trim()) {
            setStatus('error');
            setMessage('Name and API Key are required');
            return;
        }

        setStatus('loading');
        setMessage('Adding account...');

        try {
            await onAdd({
                name: name.trim(),
                apiKey: apiKey.trim(),
                baseUrl: baseUrl.trim() || 'https://api.openai.com/v1',
                organization: organization.trim() || undefined,
                isActive: false,
            });

            setStatus('success');
            setMessage('Account added successfully!');
            showToast('Account added successfully', 'success');

            setTimeout(() => {
                handleClose();
            }, 1500);
        } catch (error) {
            setStatus('error');
            setMessage(`Failed to add account: ${error}`);
            showToast(`Failed to add account: ${error}`, 'error');
        }
    };

    const handleImport = async () => {
        if (!importData.trim()) {
            setStatus('error');
            setMessage('Please paste account data to import');
            return;
        }

        setStatus('loading');
        setMessage('Importing accounts...');

        try {
            let accounts: Partial<OpenAIAccount>[] = [];
            
            // Try JSON parse
            try {
                const parsed = JSON.parse(importData);
                if (Array.isArray(parsed)) {
                    accounts = parsed;
                } else if (typeof parsed === 'object') {
                    accounts = [parsed];
                }
            } catch {
                // Try line-by-line parsing
                const lines = importData.split('\n').filter(l => l.trim());
                accounts = lines.map((line, i) => ({
                    name: `Imported Account ${i + 1}`,
                    apiKey: line.trim(),
                }));
            }

            let successCount = 0;
            for (const acc of accounts) {
                try {
                    await onAdd({
                        name: acc.name || 'Unnamed Account',
                        apiKey: acc.apiKey || '',
                        baseUrl: acc.baseUrl || 'https://api.openai.com/v1',
                        organization: acc.organization,
                        isActive: false,
                    });
                    successCount++;
                } catch (e) {
                    console.error('Failed to import account:', e);
                }
            }

            if (successCount > 0) {
                setStatus('success');
                setMessage(`Successfully imported ${successCount} account(s)`);
                showToast(`Imported ${successCount} account(s)`, 'success');
                setTimeout(() => handleClose(), 1500);
            } else {
                setStatus('error');
                setMessage('Failed to import any accounts');
            }
        } catch (error) {
            setStatus('error');
            setMessage(`Import failed: ${error}`);
            showToast(`Import failed: ${error}`, 'error');
        }
    };

    const StatusAlert = () => {
        if (status === 'idle' || !message) return null;

        const styles = {
            loading: 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800',
            success: 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800',
            error: 'bg-rose-50 dark:bg-rose-900/20 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800',
        };

        const icons = {
            loading: <Loader2 className="w-5 h-5 animate-spin" />,
            success: <CheckCircle2 className="w-5 h-5" />,
            error: <AlertCircle className="w-5 h-5" />,
        };

        return (
            <div className={cn('flex items-center gap-2 px-4 py-3 rounded-xl border mb-4', styles[status])}>
                {icons[status]}
                <span className="text-sm font-medium">{message}</span>
            </div>
        );
    };

    return (
        <>
            <button
                onClick={() => setIsOpen(true)}
                className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-xl transition-colors shadow-sm"
            >
                <Plus className="w-4 h-4" />
                Add Account
            </button>

            {isOpen && createPortal(
                <div className="fixed inset-0 z-[100] flex items-center justify-center">
                    <div 
                        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
                        onClick={handleClose}
                    />
                    
                    <div className="relative bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden">
                        {/* Header */}
                        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-800">
                            <h2 className="text-xl font-bold text-gray-900 dark:text-white">Add Account</h2>
                            <button
                                onClick={handleClose}
                                className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
                            >
                                <X className="w-5 h-5 text-gray-500" />
                            </button>
                        </div>

                        <div className="p-6">
                            {/* Tabs */}
                            <div className="flex gap-2 p-1 bg-gray-100 dark:bg-gray-800 rounded-xl mb-6">
                                <button
                                    onClick={() => setActiveTab('manual')}
                                    className={cn(
                                        'flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-lg text-sm font-medium transition-all',
                                        activeTab === 'manual'
                                            ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
                                            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    )}
                                >
                                    <Key className="w-4 h-4" />
                                    Manual
                                </button>
                                <button
                                    onClick={() => setActiveTab('import')}
                                    className={cn(
                                        'flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-lg text-sm font-medium transition-all',
                                        activeTab === 'import'
                                            ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm'
                                            : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    )}
                                >
                                    <Upload className="w-4 h-4" />
                                    Import
                                </button>
                            </div>

                            <StatusAlert />

                            {activeTab === 'manual' ? (
                                <div className="space-y-4">
                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            Account Name *
                                        </label>
                                        <input
                                            type="text"
                                            value={name}
                                            onChange={(e) => setName(e.target.value)}
                                            placeholder="e.g., Production Account"
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                    </div>

                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            API Key *
                                        </label>
                                        <input
                                            type="password"
                                            value={apiKey}
                                            onChange={(e) => setApiKey(e.target.value)}
                                            placeholder="sk-..."
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
                                        />
                                    </div>

                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            Base URL
                                        </label>
                                        <input
                                            type="text"
                                            value={baseUrl}
                                            onChange={(e) => setBaseUrl(e.target.value)}
                                            placeholder="https://api.openai.com/v1"
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                    </div>

                                    <div>
                                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            Organization (Optional)
                                        </label>
                                        <input
                                            type="text"
                                            value={organization}
                                            onChange={(e) => setOrganization(e.target.value)}
                                            placeholder="org-..."
                                            className="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        />
                                    </div>
                                </div>
                            ) : (
                                <div className="space-y-4">
                                    <p className="text-sm text-gray-600 dark:text-gray-400">
                                        Paste account data in JSON format or one API key per line:
                                    </p>
                                    <textarea
                                        value={importData}
                                        onChange={(e) => setImportData(e.target.value)}
                                        placeholder={`[\n  {\n    "name": "Account 1",\n    "apiKey": "sk-..."\n  }\n]`}
                                        className="w-full h-48 px-4 py-3 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm resize-none"
                                    />
                                </div>
                            )}

                            {/* Actions */}
                            <div className="flex gap-3 mt-6">
                                <button
                                    onClick={handleClose}
                                    className="flex-1 px-4 py-2.5 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 font-medium rounded-xl hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    onClick={activeTab === 'manual' ? handleSubmit : handleImport}
                                    disabled={status === 'loading' || status === 'success'}
                                    className="flex-1 px-4 py-2.5 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-xl transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {status === 'loading' ? (
                                        <span className="flex items-center justify-center gap-2">
                                            <Loader2 className="w-4 h-4 animate-spin" />
                                            Processing...
                                        </span>
                                    ) : activeTab === 'manual' ? (
                                        'Add Account'
                                    ) : (
                                        'Import Accounts'
                                    )}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>,
                document.body
            )}
        </>
    );
}
