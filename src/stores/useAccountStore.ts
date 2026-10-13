import { create } from 'zustand';
import type { OpenAIAccount } from '../types/account';

// NOTE: this store currently keeps accounts in memory only. The Rust backend
// commands (list_accounts, add_account, ...) are not wired up to the UI yet.

export type NewAccount = Omit<OpenAIAccount, 'id' | 'createdAt'>;

interface AccountState {
  accounts: OpenAIAccount[];
  currentAccount: OpenAIAccount | null;
  loading: boolean;
  error: string | null;
  fetchAccounts: () => Promise<void>;
  fetchCurrentAccount: () => Promise<void>;
  addAccount: (account: NewAccount) => Promise<OpenAIAccount>;
  deleteAccount: (id: string) => Promise<void>;
  deleteAccounts: (ids: string[]) => Promise<void>;
  switchAccount: (id: string) => Promise<void>;
  toggleAccountStatus: (id: string, enable: boolean, reason?: string) => Promise<void>;
  refreshQuota: (id: string) => Promise<void>;
  refreshAllQuotas: () => Promise<void>;
  exportAccounts: () => Promise<string>;
}

const nowSeconds = () => Math.floor(Date.now() / 1000);

const newId = () =>
  typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;

export const useAccountStore = create<AccountState>((set, get) => ({
  accounts: [],
  currentAccount: null,
  loading: false,
  error: null,

  fetchAccounts: async () => {
    set({ loading: false, error: null });
  },

  fetchCurrentAccount: async () => {
    const { accounts, currentAccount } = get();
    if (currentAccount && !accounts.some(a => a.id === currentAccount.id)) {
      set({ currentAccount: null });
    }
  },

  addAccount: async (input) => {
    const account: OpenAIAccount = { ...input, id: newId(), createdAt: nowSeconds() };
    set(state => {
      const accounts = [...state.accounts, account];
      return { accounts, currentAccount: state.currentAccount ?? account };
    });
    return account;
  },

  deleteAccount: async (id) => {
    await get().deleteAccounts([id]);
  },

  deleteAccounts: async (ids) => {
    const remove = new Set(ids);
    set(state => ({
      accounts: state.accounts.filter(a => !remove.has(a.id)),
      currentAccount:
        state.currentAccount && remove.has(state.currentAccount.id) ? null : state.currentAccount,
    }));
  },

  switchAccount: async (id) => {
    const target = get().accounts.find(a => a.id === id);
    if (!target) throw new Error(`Account ${id} not found`);
    if (target.disabled) throw new Error(`Account ${target.name} is disabled`);
    const lastUsed = nowSeconds();
    set(state => ({
      accounts: state.accounts.map(a =>
        a.id === id ? { ...a, isActive: true, lastUsed } : { ...a, isActive: false }
      ),
      currentAccount: { ...target, isActive: true, lastUsed },
    }));
  },

  toggleAccountStatus: async (id, enable, reason) => {
    if (!get().accounts.some(a => a.id === id)) throw new Error(`Account ${id} not found`);
    set(state => {
      const accounts = state.accounts.map(a =>
        a.id === id ? { ...a, disabled: !enable, disabledReason: enable ? undefined : reason } : a
      );
      const currentAccount = state.currentAccount
        ? accounts.find(a => a.id === state.currentAccount!.id) ?? null
        : null;
      return { accounts, currentAccount };
    });
  },

  // Quota data comes from the backend usage poller, which is not connected yet.
  refreshQuota: async (id) => {
    if (!get().accounts.some(a => a.id === id)) throw new Error(`Account ${id} not found`);
  },

  refreshAllQuotas: async () => {},

  exportAccounts: async () => JSON.stringify(get().accounts, null, 2),
}));
