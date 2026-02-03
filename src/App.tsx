import { useEffect } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/layout/Layout';
import Dashboard from './pages/Dashboard';
import Accounts from './pages/Accounts';
import Settings from './pages/Settings';
import About from './pages/About';
import { useConfigStore } from './stores/useConfigStore';

function App() {
    const { config, loadConfig, updateTheme } = useConfigStore();

    useEffect(() => {
        loadConfig();
    }, []);

    // Apply theme on mount and when config changes
    useEffect(() => {
        if (!config) return;
        
        const root = document.documentElement;
        let theme = config.theme;
        
        if (theme === 'system') {
            theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
        
        root.classList.toggle('dark', theme === 'dark');
    }, [config?.theme]);

    return (
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<Layout />}>
                    <Route index element={<Dashboard />} />
                    <Route path="accounts" element={<Accounts />} />
                    <Route path="settings" element={<Settings />} />
                    <Route path="about" element={<About />} />
                </Route>
            </Routes>
        </BrowserRouter>
    );
}

export default App;
