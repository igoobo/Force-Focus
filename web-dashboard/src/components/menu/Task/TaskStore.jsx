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

const normalizeTask = (t) => {
  if (!t) return null;
  const name = t.name || "제목 없음";
  return {
    id: t.id || t._id,
    label: name,
    appPaths: t.target_executable ? t.target_executable.split(',').filter(p => p.trim() !== "") : [],
    description: t.description || "",
    isCustom: !DEFAULT_TASK_NAMES.includes(name), 
    status: t.status || "pending"
  };
};

export const useTaskStore = create((set, get) => ({
  tasks: [],
  loading: false,

  initializeDefaultTasks: async () => {
    set({ loading: true });
    try {
      // 모든 기본 작업을 서버에 생성 요청
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
      // 생성 완료 후 데이터 다시 불러오기
      const response = await taskApi.getAll();
      const mapped = response.data.map(normalizeTask).filter(Boolean);
      set({ tasks: mapped, loading: false });
    } catch (err) {
      console.error("초기화 실패:", err);
      set({ loading: false, error: "기본 작업 초기화에 실패했습니다." });
    }
  },

  resetTasks: () => {
    set({ tasks: [], loading: false });
  },

  // 서버에서 작업 목록 가져오기
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