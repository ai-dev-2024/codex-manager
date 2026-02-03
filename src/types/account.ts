import { create } from 'zustand';

export interface OpenAIAccount {
    id: string;
    name: string;
    apiKey: string;
    baseUrl?: string;
    organization?: string;
    isActive: boolean;
    disabled?: boolean;
    disabledReason?: string;
    quota?: QuotaData;
    usage?: UsageData;
    createdAt: number;
    lastUsed?: number;
    priority?: number;
}

export interface QuotaData {
    totalRequests?: number;
    remainingRequests?: number;
    totalTokens?: number;
    remainingTokens?: number;
    resetTime?: string;
    tier?: 'free' | 'tier1' | 'tier2' | 'tier3' | 'tier4' | 'tier5';
}

export interface UsageData {
    dailyRequests: number;
    dailyTokens: number;
    monthlyRequests: number;
    monthlyTokens: number;
    lastRequestAt?: number;
}

export interface ModelQuota {
    model: string;
    percentage: number;
    resetTime?: string;
}

export interface DeviceProfile {
    machineId: string;
    macMachineId: string;
    devDeviceId: string;
    sqmId: string;
}

export interface DeviceProfileVersion {
    id: string;
    createdAt: number;
    label: string;
    profile: DeviceProfile;
    isCurrent?: boolean;
}
