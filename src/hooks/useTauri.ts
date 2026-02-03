import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function useTauri<T>(command: string, args?: Record<string, unknown>) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let isMounted = true;

    const fetchData = async () => {
      try {
        setIsLoading(true);
        const result = await invoke<T>(command, args);
        if (isMounted) {
          setData(result);
          setError(null);
        }
      } catch (err) {
        if (isMounted) {
          setError(err as Error);
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    };

    fetchData();

    return () => {
      isMounted = false;
    };
  }, [command, JSON.stringify(args)]);

  return { data, error, isLoading };
}

export function useInvoke<T>() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const invokeCommand = async (command: string, args?: Record<string, unknown>): Promise<T | null> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<T>(command, args);
      return result;
    } catch (err) {
      setError(err as Error);
      return null;
    } finally {
      setIsLoading(false);
    }
  };

  return { invokeCommand, isLoading, error };
}
