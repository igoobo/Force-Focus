import api from './axiosInstance';

export const scheduleApi = {
  create: (data) => api.post('/schedules/', data),
  getAll: () => api.get('/schedules/'),             
  getOne: (id) => api.get(`/schedules/${id}`),     
  update: (id, data) => api.put(`/schedules/${id}`, data),
  delete: (id) => api.delete(`/schedules/${id}`),
};