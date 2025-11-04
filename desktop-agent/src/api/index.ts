// 파일 위치: Force-Focus/desktop-agent/src/api/index.ts

import { User, Task, Session, Profile } from '../types';

const API_BASE_URL = '/api/v1'; // MSW가 가로챌 기본 경로

interface ApiResponse<T> {
  status: string;
  data?: T;
  event_id?: string; // POST /events 응답용
  detail?: string; // 에러 메시지용
  // 기타 공통 응답 필드가 있다면 여기에 추가
}

// --- 사용자 관련 API ---
export const fetchCurrentUser = async (): Promise<User> => {
  const response = await fetch(`${API_BASE_URL}/users/me`);
  if (!response.ok) {
    const errorData: ApiResponse<User> = await response.json();
    throw new Error(errorData.detail || 'Failed to fetch current user');
  }
  return response.json();
};

// --- Task 관련 API ---
export const fetchTasks = async (): Promise<Task[]> => {
  const response = await fetch(`${API_BASE_URL}/tasks`);
  if (!response.ok) {
    const errorData: ApiResponse<Task[]> = await response.json();
    throw new Error(errorData.detail || 'Failed to fetch tasks');
  }
  return response.json();
};

export const fetchTaskById = async (taskId: string): Promise<Task> => {
    const response = await fetch(`${API_BASE_URL}/tasks/${taskId}`);
    if (!response.ok) {
        const errorData: ApiResponse<Task> = await response.json();
        throw new Error(errorData.detail || `Failed to fetch task ${taskId}`);
    }
    return response.json();
};

// --- 세션 관련 API ---
export const fetchCurrentSession = async (): Promise<Session | null> => {
  const response = await fetch(`${API_BASE_URL}/sessions/current`);
  if (!response.ok) {
    // MSW 핸들러에서 404를 반환할 경우, 활성 세션이 없음을 의미
    if (response.status === 404) {
      return null;
    }
    const errorData: ApiResponse<Session> = await response.json();
    throw new Error(errorData.detail || 'Failed to fetch current session');
  }
  return response.json();
};

export const startSession = async (taskId?: string, goalDuration?: number): Promise<Session> => {
  const response = await fetch(`${API_BASE_URL}/sessions/start`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ task_id: taskId, goal_duration: goalDuration }),
  });
  if (!response.ok) {
    const errorData: ApiResponse<Session> = await response.json();
    throw new Error(errorData.detail || 'Failed to start session');
  }
  return response.json();
};

export const endSession = async (sessionId: string): Promise<{ success: boolean; session_id: string }> => {
  const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    // 필요하다면 body에 세션 종료와 관련된 추가 데이터 포함
    body: JSON.stringify({ status: 'ended' }) // 예시: PUT 요청 시 status를 'ended'로 보냄
  });
  if (!response.ok) {
    const errorData: ApiResponse<any> = await response.json();
    throw new Error(errorData.detail || `Failed to end session ${sessionId}`);
  }
  return response.json();
};

// --- 활동 데이터 전송 (나중에 Rust 백엔드에서 사용할 것) ---
export const postEvent = async (eventData: any): Promise<string> => { // eventData 타입은 추후 Rust 모델과 일치시킬 것
  const response = await fetch(`${API_BASE_URL}/events`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(eventData),
  });
  if (!response.ok) {
    const errorData: ApiResponse<any> = await response.json();
    throw new Error(errorData.detail || 'Failed to post event');
  }
  const result: ApiResponse<any> = await response.json();
  return result.event_id || 'no_event_id';
};