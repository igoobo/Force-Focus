import api from './axiosInstance';

export const taskApi = {
  create: (data) => api.post('/tasks/', data),
  getAll: () => api.get('/tasks/'),
  getOne: (id) => api.get(`/tasks/${id}`),
  update: (id, data) => api.put(`/tasks/${id}`, data),
  delete: (id) => api.delete(`/tasks/${id}`),
};
