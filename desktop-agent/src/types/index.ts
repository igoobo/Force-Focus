// 파일 위치: Force-Focus/desktop-agent/src/types/index.ts

// handlers.ts의 Mock 데이터 구조를 기반으로 타입 정의

export interface User {
  id: string;
  email: string;
  username: string;
  settings: {
    notifications_enabled: boolean;
    dark_mode: boolean;
  };
  blocked_apps: string[];
}

export interface Task {
  id: string;
  user_id: string;
  task_name: string;
  description: string;
  due_date: string;
  status: 'active' | 'completed' | 'pending';
  target_executable: string | null;
  target_arguments: string[];
  created_at: string;
  updated_at: string;
}

export interface Schedule {
  id: string;
  user_id: string;
  task_id: string;
  name: string;
  start_time: string; // "HH:MM"
  end_time: string;   // "HH:MM"
  days_of_week: number[]; // 1=월, 7=일
  created_at: string;
  is_active: boolean;
}

export interface Profile {
  id: string;
  user_id: string;
  profile_name: string;
  is_default: boolean;
  model_type: string;
  time_slices: Array<{ slice_index: number; rules: { [key: string]: number } }>;
  model_confidence_score: number;
  last_updated_at: string;
  custom_thresholds: { [key: string]: number };
}

export interface Session {
  id: string;
  user_id: string;
  profile_id: string;
  start_time: string; // ISO string
  status: 'active' | 'ended';
  goal_duration: number; // 분 단위
  interruption_count: number;
  task_id?: string; // 세션과 연결된 task_id (handlers.ts의 mockCurrentSession에는 없지만 추가 가능성 고려)
}