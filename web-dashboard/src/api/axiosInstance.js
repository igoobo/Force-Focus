import axios from 'axios';
import useMainStore from '../MainStore.jsx';

const api = axios.create({
  baseURL: '/api/v1',
});

// 여러 API가 동시에 401을 받을 때 alert가 여러 번 뜨는 것을 방지
let isUnauthorizedHandling = false;

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response && error.response.status === 401) {
      if (!isUnauthorizedHandling) {
        isUnauthorizedHandling = true; // 플래그 잠금
        
        alert("세션이 만료되었습니다. 다시 로그인해 주세요.");
        
        localStorage.removeItem('accessToken');
        useMainStore.getState().logout(); // Store 상태 초기화
        window.location.href = '/'; 
      }
    }
    return Promise.reject(error);
  }
);

export default api;