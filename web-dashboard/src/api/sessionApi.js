import axios from 'axios';

const api = axios.create({ baseURL: '/api/v1' });

// 요청 인터셉터를 추가하여 모든 요청에 토큰을 자동으로 포함시킵니다.
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken'); // 저장된 토큰 키 확인 필요
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
}, (error) => {
  return Promise.reject(error);
});

export const sessionApi = {
  getSessions: (limit = 100) => api.get('/sessions/', { params: { limit } }),
  getEvents: (startTime, endTime, limit = 1000) => 
    api.get('/events/', { 
      params: { 
        start_time: startTime, 
        end_time: endTime, 
        limit 
      } 
    }),
};