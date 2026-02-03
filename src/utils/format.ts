export function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function formatNumber(num: number): string {
    if (num >= 1000000) {
        return (num / 1000000).toFixed(1) + 'M';
    }
    if (num >= 1000) {
        return (num / 1000).toFixed(1) + 'K';
    }
    return num.toString();
}

export function formatPercentage(value: number, total: number): number {
    if (total === 0) return 0;
    return Math.round((value / total) * 100);
}

export function getQuotaColor(percentage: number): 'success' | 'warning' | 'error' | 'neutral' {
    if (percentage >= 50) return 'success';
    if (percentage >= 20) return 'warning';
    if (percentage > 0) return 'error';
    return 'neutral';
}

export function getQuotaColorClass(percentage: number): string {
    const color = getQuotaColor(percentage);
    switch (color) {
        case 'success':
            return 'bg-emerald-500 text-emerald-500';
        case 'warning':
            return 'bg-amber-500 text-amber-500';
        case 'error':
            return 'bg-rose-500 text-rose-500';
        default:
            return 'bg-gray-400 text-gray-400';
    }
}

export function formatTimeRemaining(dateStr: string | undefined): string {
    if (!dateStr) return 'N/A';
    const target = new Date(dateStr);
    const now = new Date();
    const diffMs = target.getTime() - now.getTime();

    if (diffMs <= 0) return 'Reset';

    const diffHrs = Math.floor(diffMs / (1000 * 60 * 60));
    const diffMins = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    if (diffHrs >= 24) {
        const days = Math.floor(diffHrs / 24);
        return `${days}d ${diffHrs % 24}h`;
    }

    return `${diffHrs}h ${diffMins}m`;
}

export function formatDate(timestamp: number | undefined): string {
    if (!timestamp) return 'Never';
    return new Date(timestamp * 1000).toLocaleString();
}

export function formatRelativeTime(timestamp: number | undefined): string {
    if (!timestamp) return 'Never';
    const now = Date.now();
    const diff = now - timestamp * 1000;
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return 'Just now';
}

export function truncate(str: string, length: number): string {
    if (str.length <= length) return str;
    return str.slice(0, length) + '...';
}

export function maskApiKey(key: string): string {
    if (key.length <= 8) return '****';
    return key.slice(0, 4) + '****' + key.slice(-4);
}
