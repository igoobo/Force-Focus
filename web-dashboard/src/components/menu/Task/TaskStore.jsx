import { create } from "zustand";
import { taskApi } from "../../../api/taskApi";

// 기본 제공되는 작업 이름 및 프로그램 목록 정의
const DEFAULT_TASK_MAPPING = {
  '코딩 작업': 'Code.exe,chrome.exe',
  '자료 조사': 'chrome.exe,msedge.exe',
  '문서 작성': 'winword.exe,notion.exe',
  '발표 자료 작성': 'powerpnt.exe',
  '학습 및 공부': 'Obsidian.exe,chrome.exe'
};

const DEFAULT_TASK_NAMES = Object.keys(DEFAULT_TASK_MAPPING);

// [수정] t.description 및 t.name을 모두 활용하여 isCustom을 더 정확하게 판별
const normalizeTask = (t) => {
  if (!t) return null;
  const name = t.name || "제목 없음";
  const desc = t.description || "";
  
  // 기본 작업 이름 목록에 포함되어 있지 않거나, 설명이 명시적으로 "사용자 추가 작업"인 경우 true
  const isCustomTask = !DEFAULT_TASK_NAMES.includes(name) || desc === "사용자 추가 작업";
  // 단, 이름은 기본 목록에 있는데 설명이 "시스템"인 경우는 명백히 false (이중 검증)
  const finalIsCustom = DEFAULT_TASK_NAMES.includes(name) && desc === "시스템 기본 제공 작업" ? false : isCustomTask;

  return {
    id: t.id || t._id,
    label: name,
    appPaths: t.target_executable ? t.target_executable.split(',').filter(p => p.trim() !== "") : [],
    description: desc,
    isCustom: finalIsCustom, 
    status: t.status || "pending"
  };
};

export const useTaskStore = create((set, get) => ({
  tasks: [],
  loading: false,

  initializeDefaultTasks: async () => {
    set({ loading: true });
    try {
      await Promise.all(
        DEFAULT_TASK_NAMES.map(name => 
          taskApi.create({
            name: name,
            description: "시스템 기본 제공 작업",
            status: "pending",
            target_executable: DEFAULT_TASK_MAPPING[name]
          })
        )
      );
      const response = await taskApi.getAll();
      const mapped = response.data.map(normalizeTask).filter(Boolean);
      set({ tasks: mapped, loading: false });
    } catch (err) {
      console.error("초기화 실패:", err);
      set({ loading: false });
    }
  },

  resetTasks: () => {
    set({ tasks: [], loading: false });
  },

  fetchTasks: async () => {
    set({ loading: true });
    try {
      const response = await taskApi.getAll();
    
      if (!response.data || response.data.length === 0) {
        await get().initializeDefaultTasks();
        return;
      }

      const mapped = response.data.map(normalizeTask).filter(Boolean);
      set({ tasks: mapped, loading: false });
    } catch (err) {
      console.error("Task Fetch Error:", err);
      set({ loading: false });
    }
  },

  addTask: async (name, paths = "") => {
    try {
      const payload = {
        name: name,
        description: "사용자 추가 작업", 
        status: "pending",
        target_executable: paths
      };
      await taskApi.create(payload);
      await get().fetchTasks(); 
    } catch (err) {
      console.error("작업 추가 실패:", err);
      throw err;
    }
  },

  // [중요 수정] 업데이트 시 description이 소실되지 않도록 페이로드에 포함
  updateTaskApps: async (id, paths) => {
    try {
      const task = get().tasks.find(t => t.id === id);
      const payload = {
        name: task.label,
        description: task.description, // 기존 설명을 유지하여 isCustom 판별 유지
        target_executable: paths.filter(p => p.trim() !== "").join(','), 
      };
      await taskApi.update(id, payload);
    } catch (err) {
      console.error(`Update failed for ${id}:`, err);
      throw err;
    }
  },

  deleteTask: async (id) => {
    try {
      await taskApi.delete(id);
      await get().fetchTasks();
    } catch (err) {
      alert("삭제 실패");
    }
  },

  setLocalPaths: (id, newPaths) => {
    set((state) => ({
      tasks: state.tasks.map(t => t.id === id ? { ...t, appPaths: newPaths } : t)
    }));
  }
}));