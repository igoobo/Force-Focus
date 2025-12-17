import axios from 'axios';

const api = axios.create({
  // FastAPI 서버 주소 (임시 주소, 추후 서버 확정 시 변경)
  baseURL: 'http://localhost:8000',
});

export default api;
