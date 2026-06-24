import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getSettings } from '../api/tauri';

export function useSettings() {
  return useQuery({
    queryKey: ['settings'],
    queryFn: getSettings,
  });
}

export function useInvalidateSettings() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ['settings'] });
}
