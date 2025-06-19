import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { servicesApi } from '../lib/api/services';

export const useServices = () => {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['services'],
    queryFn: servicesApi.getAll,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });

  const createService = useMutation({
    mutationFn: servicesApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['services'] });
    },
  });

  const updateService = useMutation({
    mutationFn: ({ id, data }: { id: string; data: any }) => 
      servicesApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['services'] });
    },
  });

  const deleteService = useMutation({
    mutationFn: servicesApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['services'] });
    },
  });

  return {
    ...query,
    createService,
    updateService,
    deleteService,
  };
};

export const useService = (id: string) => {
  return useQuery({
    queryKey: ['services', id],
    queryFn: () => servicesApi.getById(id),
    enabled: !!id,
  });
};