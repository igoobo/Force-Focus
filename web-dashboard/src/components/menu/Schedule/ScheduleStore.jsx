import { create } from "zustand";
import { scheduleApi } from "../../../api/scheduleApi";

// 서버 데이터와 프론트엔드 임시 데이터를 동기화하는 정규화 함수
const normalizeSchedule = (s) => ({
  ...s,
  name: s.name || "새 일정",
  description: s.description || "등록된 설명이 없습니다.",
  start_date: s.start_date || "2026-01-17",
  end_date: s.end_date || "2026-01-17",
  start_time: s.start_time || "00:00:00",
  end_time: s.end_time || "00:00:00",
  task_id: s.task_id || "미분류 작업"
});

export const useScheduleStore = create((set, get) => ({
  schedules: [],
  loading: false,
  error: null,

  // 1. 전체 일정 불러오기 (Read)
  fetchSchedules: async () => {
    set({ loading: true, error: null });
    try {
      const response = await scheduleApi.getAll();
      const mappedData = response.data.map(normalizeSchedule);
      set({ schedules: mappedData, loading: false });
    } catch (err) {
      console.error("Fetch Error:", err);
      set({ error: err.message, loading: false });
    }
  },

  // 2. 일정 추가 (Create)
  addSchedule: async (newSchedule) => {
    try {
      const rawStartTime = newSchedule.start_time || "09:00";
      const rawEndTime = newSchedule.end_time || "10:00";

      const scheduleDataForBackend = {
        name: newSchedule.name || "새 일정",
        task_id: newSchedule.task_id || null,
        description: newSchedule.description || "임시 데이터입니다. 추후 연동 예정입니다.",
        start_time: rawStartTime.length === 5 ? `${rawStartTime}:00` : rawStartTime,
        end_time: rawEndTime.length === 5 ? `${rawEndTime}:00` : rawEndTime,
        days_of_week: [0, 1, 2, 3, 4, 5, 6], 
        start_date: newSchedule.start_date || "2026-01-17",
        end_date: newSchedule.end_date || "2026-01-17",
        is_active: true
      };

      const response = await scheduleApi.create(scheduleDataForBackend);
      const normalizedNewItem = normalizeSchedule(response.data);
      
      set((state) => ({
        schedules: [...state.schedules, normalizedNewItem]
      }));

      // 전체 목록을 서버와 다시 한번 동기화
      await get().fetchSchedules(); 
    } catch (err) {
      const detail = err.response?.data?.detail;
      alert("일정 저장 실패: " + (typeof detail === 'object' ? JSON.stringify(detail) : (detail || err.message)));
    }
  },

  // 3. 일정 수정 (Update)
  updateSchedule: async (updatedSchedule) => {
    try {
      const { id, ...data } = updatedSchedule;
      
      const updateData = {
        ...data,
        start_time: data.start_time?.length === 5 ? `${data.start_time}:00` : data.start_time,
        end_time: data.end_time?.length === 5 ? `${data.end_time}:00` : data.end_time,
      };

      await scheduleApi.update(id, updateData);
      await get().fetchSchedules();
    } catch (err) {
      alert("일정 수정 실패: " + (err.response?.data?.detail || err.message));
    }
  },

  // 4. 일정 삭제 (Delete)
  deleteSchedule: async (id) => {
    try {
      await scheduleApi.delete(id);
      await get().fetchSchedules();
    } catch (err) {
      alert("일정 삭제 실패: " + (err.response?.data?.detail || err.message));
    }
  },
}));