import { useState, useEffect, useCallback } from 'react';
import { taskApi } from '../api/taskApi';

export const useTasks = () => {
  const [tasks, setTasks] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  // 1. READ ALL: 모든 할 일 가져오기
  const fetchTasks = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await taskApi.getAll(); // GET /tasks/
      setTasks(response.data);
    } catch (err) {
      setError(err.response?.data?.detail || "할 일을 불러오는 데 실패했습니다.");
    } finally {
      setLoading(false);
    }
  }, []);

  // 2. CREATE: 할 일 추가
  const addTask = async (taskData) => {
    try {
      const response = await taskApi.create(taskData); // POST /tasks/
      setTasks((prev) => [...prev, response.data]);
      return response.data;
    } catch (err) {
      setError("할 일 추가 중 오류가 발생했습니다.");
      throw err;
    }
  };

  // 3. UPDATE: 할 일 수정
  const updateTask = async (id, updateData) => {
    try {
      const response = await taskApi.update(id, updateData); // PUT /tasks/{task_id}
      setTasks((prev) =>
        prev.map((t) => (t.id === id ? response.data : t))
      );
      return response.data;
    } catch (err) {
      setError("할 일 수정 중 오류가 발생했습니다.");
      throw err;
    }
  };

  // 4. DELETE: 할 일 삭제
  const removeTask = async (id) => {
    try {
      await taskApi.delete(id); // DELETE /tasks/{task_id}
      setTasks((prev) => prev.filter((t) => t.id !== id));
    } catch (err) {
      // 삭제 실패 시 (404 등) 처리
      setError(err.response?.data?.detail || "할 일 삭제 중 오류가 발생했습니다.");
      throw err;
    }
  };

  // 컴포넌트 마운트 시 자동 로드
  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  return {
    tasks,
    loading,
    error,
    addTask,
    updateTask,
    removeTask,
    refreshTasks: fetchTasks,
  };
};