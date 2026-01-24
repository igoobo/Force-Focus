import { create } from "zustand";
import { taskApi } from "../../../api/taskApi";

// 기본 제공되는 작업 이름 및 프로그램 목록 정의
const DEFAULT_APP_MAPPING = {
  '코딩 작업': ['Code.exe', 'chrome.exe'],
  '자료 조사': ['chrome.exe', 'msedge.exe'],
  '문서 작성': ['winword.exe', 'notion.exe'],
  '발표 자료 작성': ['powerpnt.exe'],
  '학습 및 공부': ['Obsidian.exe', 'chrome.exe']
};

const normalizeTask = (t) => {
  const name = t.name || "제목 없음";
  const isCustom = !Object.keys(DEFAULT_APP_MAPPING).includes(name);
  let appPaths = [];
  if (t.target_executable) {
    appPaths = t.target_executable.split(',').filter(p => p.trim() !== "");
  } else if (!isCustom) {
    appPaths = DEFAULT_APP_MAPPING[name] || [];
  }

  return {
    id: t.id || t._id,
    label: name,
    appPaths: appPaths,
    description: t.description || "",
    isCustom: isCustom,
    status: t.status || "pending"
  };
};

export const useTaskStore = create((set, get) => ({
  tasks: [],
  loading: false,

  // 서버에서 작업 목록 가져오기
  fetchTasks: async () => {
    set({ loading: true });
    try {
      const response = await taskApi.getAll();
      const mapped = response.data.map(normalizeTask);
      set({ tasks: mapped, loading: false });
    } catch (err) {
      console.error("Task Fetch Error:", err);
      set({ loading: false });
    }
  },

  // 새 작업 추가 (사용자 정의 작업은 항상 isCustom: true)
  addTask: async (name) => {
    try {
      const payload = {
        name: name,
        description: "사용자 추가 작업",
        status: "pending",
        target_executable: "" 
      };
      await taskApi.create(payload);
      await get().fetchTasks(); // 목록 새로고침
    } catch (err) {
      alert("작업 추가 실패");
    }
  },

  // 특정 작업의 프로그램 리스트 업데이트 (저장하기 버튼용)
  updateTaskApps: async (id, paths) => {
    try {
      const task = get().tasks.find(t => t.id === id);
      const payload = {
        name: task.label,
        target_executable: paths.filter(p => p.trim() !== "").join(','), 
      };
      await taskApi.update(id, payload);
    } catch (err) {
      console.error(`Update failed for ${id}:`, err);
      throw err;
    }
  },

  // 작업 삭제 (isCustom이 true인 경우만 호출됨)
  deleteTask: async (id) => {
    try {
      await taskApi.delete(id);
      await get().fetchTasks();
    } catch (err) {
      alert("삭제 실패");
    }
  },

  // 로컬 상태 업데이트 (입력 중인 값을 UI에 즉시 반영)
  setLocalPaths: (id, newPaths) => {
    set((state) => ({
      tasks: state.tasks.map(t => t.id === id ? { ...t, appPaths: newPaths } : t)
    }));
  }
}));