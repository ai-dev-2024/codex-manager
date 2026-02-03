import { Outlet, Link, useLocation } from 'react-router-dom';
import { 
    LayoutDashboard, 
    Users, 
    Settings, 
    Info,
    Sun,
    Moon,
    Command
} from 'lucide-react';
import { cn } from '../../utils/cn';
import { useConfigStore } from '../../stores/useConfigStore';
import ToastContainer from '../common/ToastContainer';

const navItems = [
    { path: '/', label: 'Dashboard', icon: LayoutDashboard },
    { path: '/accounts', label: 'Accounts', icon: Users },
    { path: '/settings', label: 'Settings', icon: Settings },
    { path: '/about', label: 'About', icon: Info },
];

function Sidebar() {
    const location = useLocation();
    const { config, updateTheme } = useConfigStore();

    const isActive = (path: string) => {
        if (path === '/') {
            return location.pathname === '/';
        }
        return location.pathname.startsWith(path);
    };

    const toggleTheme = () => {
        const newTheme = config.theme === 'light' ? 'dark' : 'light';
        updateTheme(newTheme);
    };

    return (
        <aside className="w-64 h-screen bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 flex flex-col">
            {/* Logo */}
            <div className="p-6 border-b border-gray-200 dark:border-gray-800">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center shadow-lg">
                        <Command className="w-6 h-6 text-white" />
                    </div>
                    <div>
                        <h1 className="text-lg font-bold text-gray-900 dark:text-white">Codex</h1>
                        <p className="text-xs text-gray-500 dark:text-gray-400">Manager</p>
                    </div>
                </div>
            </div>

            {/* Navigation */}
            <nav className="flex-1 p-4 space-y-1">
                {navItems.map((item) => {
                    const Icon = item.icon;
                    return (
                        <Link
                            key={item.path}
                            to={item.path}
                            className={cn(
                                'flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all',
                                isActive(item.path)
                                    ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400'
                                    : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-gray-200'
                            )}
                        >
                            <Icon className="w-5 h-5" />
                            {item.label}
                        </Link>
                    );
                })}
            </nav>

            {/* Bottom Actions */}
            <div className="p-4 border-t border-gray-200 dark:border-gray-800">
                <button
                    onClick={toggleTheme}
                    className="flex items-center gap-3 px-4 py-3 w-full rounded-xl text-sm font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800 transition-all"
                >
                    {config.theme === 'light' ? (
                        <>
                            <Moon className="w-5 h-5" />
                            Dark Mode
                        </>
                    ) : (
                        <>
                            <Sun className="w-5 h-5" />
                            Light Mode
                        </>
                    )}
                </button>
            </div>
        </aside>
    );
}

export default function Layout() {
    return (
        <div className="h-screen flex bg-gray-50 dark:bg-gray-950">
            <Sidebar />
            <main className="flex-1 overflow-hidden flex flex-col">
                <div className="flex-1 overflow-auto p-8">
                    <Outlet />
                </div>
            </main>
            <ToastContainer />
        </div>
    );
}
