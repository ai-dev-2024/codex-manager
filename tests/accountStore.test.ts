import { beforeEach, describe, expect, it } from 'vitest';
import { useAccountStore, type NewAccount } from '../src/stores/useAccountStore';

const input = (name: string): NewAccount => ({
  name,
  apiKey: `sk-${name}-0000000000`,
  isActive: false,
});

const store = () => useAccountStore.getState();

describe('useAccountStore', () => {
  beforeEach(() => {
    useAccountStore.setState({ accounts: [], currentAccount: null, loading: false, error: null });
  });

  it('adds accounts with unique ids and selects the first as current', async () => {
    const a = await store().addAccount(input('alpha'));
    const b = await store().addAccount(input('bravo'));
    expect(a.id).not.toBe(b.id);
    expect(typeof a.createdAt).toBe('number');
    expect(store().accounts).toHaveLength(2);
    expect(store().currentAccount?.id).toBe(a.id);
  });

  it('switches the active account and records lastUsed', async () => {
    const a = await store().addAccount(input('alpha'));
    const b = await store().addAccount(input('bravo'));
    await store().switchAccount(b.id);
    const accounts = store().accounts;
    expect(store().currentAccount?.id).toBe(b.id);
    expect(accounts.find(x => x.id === b.id)?.isActive).toBe(true);
    expect(accounts.find(x => x.id === a.id)?.isActive).toBe(false);
    expect(accounts.find(x => x.id === b.id)?.lastUsed).toBeTypeOf('number');
  });

  it('refuses to switch to unknown or disabled accounts', async () => {
    const a = await store().addAccount(input('alpha'));
    await expect(store().switchAccount('missing')).rejects.toThrow(/not found/);
    await store().toggleAccountStatus(a.id, false, 'manual');
    await expect(store().switchAccount(a.id)).rejects.toThrow(/disabled/);
  });

  it('toggles disabled state and reason', async () => {
    const a = await store().addAccount(input('alpha'));
    await store().toggleAccountStatus(a.id, false, 'quota exhausted');
    expect(store().accounts[0]).toMatchObject({ disabled: true, disabledReason: 'quota exhausted' });
    expect(store().currentAccount?.disabled).toBe(true);
    await store().toggleAccountStatus(a.id, true);
    expect(store().accounts[0]).toMatchObject({ disabled: false, disabledReason: undefined });
  });

  it('deletes single and multiple accounts and clears a deleted current account', async () => {
    const a = await store().addAccount(input('alpha'));
    const b = await store().addAccount(input('bravo'));
    const c = await store().addAccount(input('charlie'));
    await store().deleteAccount(b.id);
    expect(store().accounts.map(x => x.id)).toEqual([a.id, c.id]);
    await store().deleteAccounts([a.id, c.id]);
    expect(store().accounts).toEqual([]);
    expect(store().currentAccount).toBeNull();
  });

  it('exports accounts as JSON', async () => {
    await store().addAccount(input('alpha'));
    const parsed = JSON.parse(await store().exportAccounts());
    expect(parsed).toHaveLength(1);
    expect(parsed[0].name).toBe('alpha');
  });
});
