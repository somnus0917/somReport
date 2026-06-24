import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getToday,
  getRecordingState,
  getDailyUsage,
  updateActivity,
  deleteActivity,
} from '../api/tauri';

export function useToday() {
  return useQuery({
    queryKey: ['today'],
    queryFn: getToday,
  });
}

export function useRecordingState() {
  return useQuery({
    queryKey: ['recordingState'],
    queryFn: getRecordingState,
  });
}

export function useDailyUsage() {
  return useQuery({
    queryKey: ['dailyUsage'],
    queryFn: getDailyUsage,
  });
}

export function useUpdateActivity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: updateActivity,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['today'] });
    },
  });
}

export function useDeleteActivity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: deleteActivity,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['today'] });
    },
  });
}
