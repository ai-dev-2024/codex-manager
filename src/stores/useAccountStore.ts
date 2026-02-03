import { create } from 'zustand';

interface Account {
  id: string;
  name: string;
  apiKey: string;
  isActive: boolean;
  createdAt: string;
}

interface AccountState {
  accounts: Account[];
  currentAccount: Account | null;
  isLoading: boolean;
  error: string | null;
  fetchAccounts: () => Promise<void>;
  fetchCurrentAccount: () => Promise<void>;
  addAccount: (account: Omit<Account, 'id' | 'createdAt'>) => Promise<void>;
  deleteAccount: (id: string) => Promise<void>;
  switchAccount: (id: string) => Promise<void>;
}

export const useAccountStore = create<AccountState>((set, get) => ({
  accounts: [],
  currentAccount: null,
  isLoading: false,
  error: null,

  fetchAccounts: async () => {
    set({ isLoading: true, error: null });
    try {
      // TODO: Implement Tauri invoke
      // const accounts = await invoke<Account[]>('list_accounts');
      // set({ accounts, isLoading: false });
      set({ isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  fetchCurrentAccount: async () => {
    try {
      // TODO: Implement Tauri invoke
      // const account = await invoke<Account | null>('get_current_account');
      // set({ currentAccount: account });
    } catch (error) {
      console.error('Failed to fetch current account:', error);
    }
  },

  addAccount: async (account) => {
    set({ isLoading: true, error: null });
    try {
      // TODO: Implement Tauri invoke
      // await invoke('add_account', { account });
      await get().fetchAccounts();
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  deleteAccount: async (id) => {
    set({ isLoading: true, error: null });
    try {
      // TODO: Implement Tauri invoke
      // await invoke('delete_account', { id });
      await get().fetchAccounts();
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  switchAccount: async (id) => {
    try {
      // TODO: Implement Tauri invoke
      // await invoke('switch_account', { id });
      await get().fetchCurrentAccount();
    } catch (error) {
      console.error('Failed to switch account:', error);
    }
  },
}));
