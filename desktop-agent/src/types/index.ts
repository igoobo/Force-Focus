// 파일 위치: Force-Focus/desktop-agent/src/types/index.ts
// 프론트엔드 ↔ Rust 백엔드 공유 타입 정의

// Rust의 lib.rs Task 구조체와 일치
export interface Task {
  id: string;
  user_id: string;
  task_name: string;
  description: string;
  due_date: string;
  status: string; // Rust 백엔드는 자유 String (F-6 수정: union type → string)
  target_executable: string;
  target_arguments: string[];
  created_at: string;
  updated_at: string;
}

// Rust의 lib.rs ActiveSessionInfo와 일치
export interface ActiveSessionInfo {
  session_id: string;
  task_id: string | null;
  start_time_s: number;
}

// F-6: User, Profile, Session (MSW 전용) 삭제
// F-6: Schedule.days_of_week 인덱스 불일치 해소 (0-based로 통일)
export interface Schedule {
  id: string;
  user_id: string;
  task_id: string | null;
  name: string;
  start_time: string;
  end_time: string;
  days_of_week: number[]; // 0=월, 6=일 (Rust Vec<u8>과 일치)
  start_date: string | null;
  is_active: boolean;
}