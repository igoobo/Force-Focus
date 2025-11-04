// 파일 위치: Force-Focus/desktop-agent/src/mocks/handlers.ts
import { HttpResponse, http, delay } from 'msw';
import { User, Task, Schedule, Profile, Session } from '../types'; // types에서 인터페이스 임포트

const API_BASE_URL = '/api/v1';

// --- 가짜 데이터 정의 ---
export const mockDefaultProfile: Profile = {
  id: 'profile-default-desktop',
  user_id: 'desktop-user-123',
  profile_name: '기본 집중 프로필',
  is_default: true,
  model_type: 'focus_optimization_v1',
  time_slices: [
    { slice_index: 0, rules: { allowed_app_groups: 1, notification_control: 1 } },
  ],
  model_confidence_score: 0.85,
  last_updated_at: '2023-10-26T00:00:00Z',
  custom_thresholds: {
    interruption_sensitivity: 0.7,
  },
};

export const mockCurrentUser: User = {
  id: 'desktop-user-123',
  email: 'user@example.com',
  username: 'desktop_user',
  settings: {
    notifications_enabled: true,
    dark_mode: true,
  },
  blocked_apps: ['explorer.exe', 'steam.exe'],
};

// --- 가짜 할 일(Task) 데이터 정의 ---
export const mockTasks: Task[] = [
  {
    id: 'task-coding-session',
    user_id: 'desktop-user-123',
    task_name: '코딩 세션 진행',
    description: 'Force-Focus 데스크톱 앱 프런트엔드 개발',
    due_date: '2023-12-31T23:59:59Z',
    status: 'active',
    target_executable: 'vscode.exe',
    target_arguments: [],
    created_at: '2023-10-26T10:00:00Z',
    updated_at: '2023-10-26T10:00:00Z',
  },
  {
    id: 'task-report-writing',
    user_id: 'desktop-user-123',
    task_name: '주간 보고서 작성',
    description: '지난 주 작업 내용 정리 및 보고서 초안 작성',
    due_date: '2023-11-03T18:00:00Z',
    status: 'pending',
    target_executable: 'word.exe',
    target_arguments: [],
    created_at: '2023-10-25T09:00:00Z',
    updated_at: '2023-10-25T09:00:00Z',
  },
];
// ------------------------------------

// --- 가짜 스케줄 데이터 정의 (Task ID 참조 업데이트) ---
export const mockSchedules: Schedule[] = [
  {
    id: 'schedule-coding-mon-wed',
    user_id: 'desktop-user-123',
    task_id: 'task-coding-session',
    name: '월/수 코딩 집중',
    start_time: '09:00',
    end_time: '12:00',
    days_of_week: [1, 3], // 월요일, 수요일
    created_at: '2023-10-20T08:00:00Z',
    is_active: true,
  },
];
// ------------------------------------
// --- Mock 세션 상태를 동적으로 관리하기 위한 변수 ---
let activeUserSession: Session | null = null;

// --- MSW 핸들러 정의 ---
export const handlers = [
  // 사용자 로그인 (Mock)
  http.post(`${API_BASE_URL}/user/login`, async ({ request }) => {
    await delay(500); // 네트워크 지연 시뮬레이션
    const { email, password } = await request.json() as any;

    if (email === 'user@example.com' && password === 'password123') {
      return HttpResponse.json({
        message: '로그인 성공',
        token: 'mock-jwt-token-123',
        user: mockCurrentUser,
      }, { status: 200 });
    }
    return HttpResponse.json({ message: '잘못된 이메일 또는 비밀번호' }, { status: 401 });
  }),

  // 현재 사용자 정보 조회
  http.get(`${API_BASE_URL}/users/me`, async () => {
    await delay(300);
    return HttpResponse.json(mockCurrentUser, { status: 200 });
  }),

  // 모든 Task 조회
  http.get(`${API_BASE_URL}/tasks`, async () => {
    await delay(300);
    return HttpResponse.json(mockTasks, { status: 200 });
  }),

  // 특정 Task ID로 조회
  http.get(`${API_BASE_URL}/tasks/:taskId`, async ({ params }) => {
    await delay(300);
    const { taskId } = params;
    const task = mockTasks.find(t => t.id === taskId);
    if (task) {
      return HttpResponse.json(task, { status: 200 });
    }
    return HttpResponse.json({ detail: 'Task not found' }, { status: 404 });
  }),

  // 모든 Schedule 조회
  http.get(`${API_BASE_URL}/schedules`, async () => {
    await delay(300);
    return HttpResponse.json(mockSchedules, { status: 200 });
  }),

  // 기본 프로필 조회
  http.get(`${API_BASE_URL}/profiles/default`, async () => {
    await delay(300);
    return HttpResponse.json(mockDefaultProfile, { status: 200 });
  }),

  // 현재 활성 세션 조회 (동적 처리)
  http.get(`${API_BASE_URL}/sessions/current`, async () => {
    await delay(500);
    if (activeUserSession && activeUserSession.status === 'active') {
      // 현재 시간 기준으로 경과 시간 업데이트 (Mock)
      const updatedSession = { ...activeUserSession };
      // start_time을 기준으로 경과 시간을 계산하여 클라이언트에서 타이머가 자연스럽게 이어지도록 함
      return HttpResponse.json(updatedSession, { status: 200 });
    }
    // 활성 세션이 없으면 404 반환
    return HttpResponse.json({ detail: 'No active session found' }, { status: 404 });
  }),

  // 세션 시작 (동적 처리)
  http.post(`${API_BASE_URL}/sessions/start`, async ({ request }) => {
    await delay(700);
    const { task_id, goal_duration } = await request.json() as any;

    if (activeUserSession && activeUserSession.status === 'active') {
      return HttpResponse.json({ detail: 'Another session is already active.' }, { status: 409 }); // Conflict
    }

    const newSession: Session = {
      id: `session-${Date.now()}`,
      user_id: mockCurrentUser.id,
      profile_id: mockDefaultProfile.id, // 기본 프로필 사용
      task_id: task_id || 'task-coding-session', // task_id가 없으면 기본값
      start_time: new Date().toISOString(),
      status: 'active',
      goal_duration: goal_duration || 60, // 기본 60분
      interruption_count: 0,
    };
    activeUserSession = newSession; // 활성 세션으로 설정
    return HttpResponse.json(newSession, { status: 200 });
  }),

  // 세션 종료 (동적 처리)
  http.put(`${API_BASE_URL}/sessions/:sessionId`, async ({ params }) => {
    await delay(500);
    const { sessionId } = params;

    if (activeUserSession && activeUserSession.id === sessionId && activeUserSession.status === 'active') {
      activeUserSession = null; // 활성 세션 종료
      return HttpResponse.json({ success: true, session_id: sessionId }, { status: 200 });
    }
    return HttpResponse.json({ detail: 'Active session not found or already ended.' }, { status: 404 });
  }),

  // 이벤트 전송 (Mock)
  http.post(`${API_BASE_URL}/events`, async () => {
    await delay(200);
    return HttpResponse.json({ status: 'success', event_id: `event-${Date.now()}` }, { status: 200 });
  }),
];