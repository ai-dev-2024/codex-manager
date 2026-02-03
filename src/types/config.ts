export type SchedulingMode = 'round-robin' | 'least-used' | 'priority' | 'random' | 'weighted';
export type ThemeMode = 'light' | 'dark' | 'system';

export interface ProxyConfig {
    enabled: boolean;
    port: number;
    host: string;
    apiKey: string;
    autoStart: boolean;
    requestTimeout: number;
    enableLogging: boolean;
    upstreamProxy?: {
        enabled: boolean;
        url: string;
    };
    scheduling: {
        mode: SchedulingMode;
        maxRetries: number;
        retryDelay: number;
        healthCheckInterval: number;
    };
    rateLimiting: {
        enabled: boolean;
        requestsPerMinute: number;
        tokensPerMinute: number;
    };
}

export interface AppConfig {
    language: string;
    theme: ThemeMode;
    autoSwitch: boolean;
    autoRefresh: boolean;
    refreshInterval: number;
    defaultExportPath?: string;
    proxy: ProxyConfig;
    notifications: {
        enabled: boolean;
        quotaAlerts: boolean;
        errorAlerts: boolean;
        successAlerts: boolean;
    };
    appearance: {
        compactMode: boolean;
        showQuotaBars: boolean;
        animateCharts: boolean;
    };
}

export const defaultConfig: AppConfig = {
    language: 'en',
    theme: 'system',
    autoSwitch: false,
    autoRefresh: true,
    refreshInterval: 5,
    proxy: {
        enabled: false,
        port: 8080,
        host: '127.0.0.1',
        apiKey: '',
        autoStart: false,
        requestTimeout: 120,
        enableLogging: true,
        upstreamProxy: {
            enabled: false,
            url: '',
        },
        scheduling: {
            mode: 'round-robin',
            maxRetries: 3,
            retryDelay: 1000,
            healthCheckInterval: 60,
        },
        rateLimiting: {
            enabled: true,
            requestsPerMinute: 60,
            tokensPerMinute: 100000,
        },
    },
    notifications: {
        enabled: true,
        quotaAlerts: true,
        errorAlerts: true,
        successAlerts: false,
    },
    appearance: {
        compactMode: false,
        showQuotaBars: true,
        animateCharts: true,
    },
};
