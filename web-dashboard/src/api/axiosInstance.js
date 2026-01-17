import axios from 'axios';

const api = axios.create({
  baseURL: '/api/v1', 
});

// 인터셉터를 사용하여 모든 요청에 토큰 자동 포함
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export default api;