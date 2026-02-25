import axios from 'axios';
import useMainStore from '../MainStore.jsx';

const authApi = axios.create({
  baseURL: '/',
});

authApi.interceptors.request.use((config) => {
  const token = localStorage.getItem('accessToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

authApi.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response && error.response.status === 401) {
      alert("세션이 만료되었습니다. 다시 로그인해 주세요.");
      useMainStore.getState().logout(); // Store에 추가한 logout 실행
    }
    return Promise.reject(error);
  }
);

export default authApi;