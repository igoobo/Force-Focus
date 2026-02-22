import api from './axiosInstance';

export const sessionApi = {
  getSessions: (limit = 100) => api.get('/sessions/', { params: { limit } }),
  getEvents: (startTime, endTime, limit = 1000) => 
    api.get('/events/', { 
      params: { start_time: startTime, end_time: endTime, limit } 
    }),
};