import { create } from 'zustand';
import { AppConfig, defaultConfig } from '../types/config';

interface ConfigState {
    config: AppConfig;
    loading: boolean;
    error: string | null;

    // Actions
    loadConfig: () => Promise<void>;
    saveConfig: (config: AppConfig) => Promise<void>;
    updateTheme: (theme: AppConfig['theme']) => void;
    updateLanguage: (language: string) => void;
    updateProxy: (proxy: Partial<AppConfig['proxy']>) => void;
    updateNotifications: (notifications: Partial<AppConfig['notifications']>) => void;
    updateAppearance: (appearance: Partial<AppConfig['appearance']>) => void;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
    config: defaultConfig,
    loading: false,
    error: null,

    loadConfig: async () => {
        set({ loading: true, error: null });
        try {
            // Load from localStorage or use default
            const stored = localStorage.getItem('codex-manager-config');
            if (stored) {
                const parsed = JSON.parse(stored);
                set({ config: { ...defaultConfig, ...parsed }, loading: false });
            } else {
                set({ config: defaultConfig, loading: false });
            }
        } catch (error) {
            set({ error: String(error), loading: false });
        }
    },

    saveConfig: async (config) => {
        set({ loading: true, error: null });
        try {
            localStorage.setItem('codex-manager-config', JSON.stringify(config));
            set({ config, loading: false });
        } catch (error) {
            set({ error: String(error), loading: false });
            throw error;
        }
    },

    updateTheme: (theme) => {
        const { config, saveConfig } = get();
        const newConfig = { ...config, theme };
        set({ config: newConfig });
        localStorage.setItem('codex-manager-config', JSON.stringify(newConfig));
        
        // Apply theme immediately
        const root = document.documentElement;
        if (theme === 'system') {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            root.classList.toggle('dark', prefersDark);
        } else {
            root.classList.toggle('dark', theme === 'dark');
        }
    },

    updateLanguage: (language) => {
        const { config } = get();
        const newConfig = { ...config, language };
        set({ config: newConfig });
        localStorage.setItem('codex-manager-config', JSON.stringify(newConfig));
    },

    updateProxy: (proxy) => {
        const { config } = get();
        const newConfig = { 
            ...config, 
            proxy: { ...config.proxy, ...proxy } 
        };
        set({ config: newConfig });
        localStorage.setItem('codex-manager-config', JSON.stringify(newConfig));
    },

    updateNotifications: (notifications) => {
        const { config } = get();
        const newConfig = { 
            ...config, 
            notifications: { ...config.notifications, ...notifications } 
        };
        set({ config: newConfig });
        localStorage.setItem('codex-manager-config', JSON.stringify(newConfig));
    },

    updateAppearance: (appearance) => {
        const { config } = get();
        const newConfig = { 
            ...config, 
            appearance: { ...config.appearance, ...appearance } 
        };
        set({ config: newConfig });
        localStorage.setItem('codex-manager-config', JSON.stringify(newConfig));
    },
}));
