import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
// --- 1. 타입 정의 ---
// (types.ts 또는 유사 파일에서 가져오는 것이 좋으나, 여기서는 직접 정의)
interface Task {
  id: string;
  task_name: string;
  // ... (handlers.ts에 정의된 다른 Task 필드들)
}

// Rust의 lib.rs/storage_manager.rs와 동일한 구조
interface ActiveSessionInfo {
  session_id: string;
  task_id: string | null;
  start_time_s: number; // Unix timestamp (seconds)
}

// 부모(App.tsx)로부터 받는 Props
interface MainViewProps {
  onLogout: () => void;
}

// '기본 태스크'를 위한 특수 식별자
const BASIC_TASK_ID = "__BASIC_TASK__";

/**
 * 로그인 후 표시되는 메인 UI.
 * 세션 관리(시작, 종료, 타이머) 및 Task 조회를 담당합니다.
 */
const MainView: React.FC<MainViewProps> = ({ onLogout }) => {
  // --- 2. 상태 관리 ---
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  
  // '활성 세션'의 상태를 관리합니다.
  const [activeSession, setActiveSession] = useState<ActiveSessionInfo | null>(null);
  
  // 타이머 표시를 위한 경과 시간 (초)
  const [elapsedTime, setElapsedTime] = useState<number>(0);

  const [error, setError] = useState<string | null>(null);

  // --- 3. 데이터 조회 (Mock API) ---
  // 컴포넌트 마운트 시 Mock API에서 Task 목록을 가져옵니다.
  useEffect(() => {
    const fetchTasks = async () => {
      try {
        // msw가 가로챌 API 경로 (handlers.ts에 정의됨)
        // (주의: Rust가 아닌 React의 fetch는 msw에 의해 가로채집니다)
        const data: Task[] = await invoke('get_tasks');

        setTasks(data);

        // Task 목록 로딩이 완료된 후에 기본 선택 ID를 설정
        // 이 시점에 <select>는 <option> 목록을 모두 가지고 있으므로,
        // React 상태와 DOM 상태가 일치
        setSelectedTaskId(BASIC_TASK_ID);
        
      } catch (e: any) {
        setError(e.message || 'Failed to load data');
      }
    };

    // 3.2 [추가] Task 3.7: 'Stale Session' (꼬인 세션) 해결
    //      앱 로드 시 '현재 세션'을 1회 PULL하여 UI 즉시 동기화
    const fetchCurrentSession = async () => {
       try {
        // [수정] core.invoke 사용
        const sessionInfo: ActiveSessionInfo | null = await invoke('get_current_session_info');
        if (sessionInfo) {
          setActiveSession(sessionInfo); // [!] 꼬인 세션 복원
          // [!] (타이머 시작은 4단계 'listen'이 처리)
        }
      } catch (e: any) {
         setError(e.toString());
      }
    };

    fetchTasks();
    fetchCurrentSession();
  }, []); // 마운트 시 1회 실행

  // --- [수정] 4. 타이머 로직 (Task 4.12: Rust PUSH 수신) ---
  useEffect(() => {
    let unlistenTick: (() => void) | null = null;
    
    const setupListener = async () => {
      try {
        // [!] 'widget'과 'main' 창 모두 동일한 이벤트를 수신 (ACL 필요)
        // [수정] event.listen 사용
        const unlistenTickFn = await listen<number>("widget-tick", (e) => {
          setElapsedTime(e.payload); // Rust가 보낸 경과 시간(u64)으로 상태 업데이트
        });
        unlistenTick = unlistenTickFn;
      } catch (e: any) {
         setError(e.toString());
      }
    };
    setupListener();

    // [삭제] React의 타이머 로직 (setInterval) 완전 삭제
    // useEffect(() => { ... }, [activeSession]);

    return () => {
      if (unlistenTick) unlistenTick();
    };
  }, []); // 'listen'은 마운트 시 1회만

  // --- 5. Rust 커맨드 연결 (세션 시작) ---
  const handleStartSession = useCallback(async () => {
    // selectedTaskId가 'BASIC_TASK_ID'인 경우, Rust로 null을 전송
    const taskIdToSend = selectedTaskId === BASIC_TASK_ID ? null : selectedTaskId;

    setError(null);
    try {
      // taskId: taskIdToSend (null 가능)
      const sessionInfo: ActiveSessionInfo = await invoke('start_session', {
        taskId: taskIdToSend, 
        goalDuration: 60,
      });
      setActiveSession(sessionInfo);
    } catch (e: any) {
      setError(e.toString());
    }
  }, [selectedTaskId]); // selectedTaskId가 변경될 때마다 함수 재생성

  // --- 6. Rust 커맨드 연결 (세션 종료) ---
  const handleEndSession = useCallback(async () => {
    setError(null);
    try {
      // (Must-have 7) 세션 평가 점수를 임시로 5점으로 하드코딩
      await invoke('end_session', {
        userEvaluationScore: 5, 
      });
      
      // 세션 상태를 비활성화합니다.
      setActiveSession(null);
      setElapsedTime(0); // (Rust PUSH('widget-tick', 0)이 1초 안에 덮어쓸 것임)
    } catch (e: any) {
      setError(e.toString());
    }
  }, []);

  // --- 7. 헬퍼 함수 (시간 포맷팅) ---
  const formatTime = (seconds: number): string => {
    const h = Math.floor(seconds / 3600).toString().padStart(2, '0');
    const m = Math.floor((seconds % 3600) / 60).toString().padStart(2, '0');
    const s = (seconds % 60).toString().padStart(2, '0');
    return `${h}:${m}:${s}`;
  };

  //'기본 태스크' 선택 시 "Task 없음"을, 그 외에는 Task 이름을 표시
  // [수정] 'activeSession' (Optimistic Update) 대신 'elapsedTime' (PUSH)을 기준으로 UI 분기
  const isSessionActive = elapsedTime > 0; 
  const currentTaskName = activeSession?.task_id
    ? (tasks.find(t => t.id === activeSession.task_id)?.task_name || '알 수 없는 작업')
    : '기본 집중 (Task 없음)';

  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif' }}>
      <button onClick={onLogout} style={{ float: 'right' }}>로그아웃</button>
      <h1 style={{ marginTop: 0 }}>Force-Focus</h1>

      {error && <div style={{ color: 'red', marginBottom: '10px' }}>{error}</div>}

      {/* 세션 상태에 따라 UI 분기 */}
      {isSessionActive ? (
        // --- 세션 활성 시 (Must-have 4, 5) ---
        <div style={{ border: '2px solid green', padding: '15px', borderRadius: '8px' }}>
          <h2 style={{ marginTop: 0 }}>집중 세션 진행 중</h2>
          <p><strong>작업:</strong> {currentTaskName}</p>
          <div style={{ fontSize: '32px', fontWeight: 'bold', margin: '10px 0' }}>
            {formatTime(elapsedTime)}
          </div>
          <button 
            onClick={handleEndSession}
            style={{ backgroundColor: 'red', color: 'white', padding: '10px 15px', border: 'none', borderRadius: '5px' }}
          >
            세션 종료
          </button>
          {/* (Must-have 5: 세션 시간 늘리기 버튼은 여기에 추가) */}
        </div>
      ) : (
        // --- 세션 비활성 시 (Must-have 3) ---
        <div style={{ border: '1px solid #ccc', padding: '15px', borderRadius: '8px' }}>
          <h2 style={{ marginTop: 0 }}>새 세션 시작하기</h2>
          <label htmlFor="task-select" style={{ display: 'block', marginBottom: '5px' }}>
            작업 선택:
          </label>
          <select 
            id="task-select"
            value={selectedTaskId || ''}
            onChange={(e) => setSelectedTaskId(e.target.value)}
            style={{ width: '100%', padding: '8px', marginBottom: '15px' }}
          >

            {/* '기본 태스크' 옵션 */}
            <option value={BASIC_TASK_ID}>-- 기본 집중 (Task 없음) --</option>

            {tasks.length === 0 && <option>작업 목록 로딩 중...</option>}
            {tasks.map(task => (
              <option key={task.id} value={task.id}>
                {task.task_name}
              </option>
            ))}
          </select>
          <button 
            onClick={handleStartSession}
            disabled={!selectedTaskId}
            style={{ backgroundColor: 'green', color: 'white', padding: '10px 15px', border: 'none', borderRadius: '5px' }}
          >
            세션 시작
          </button>
        </div>
      )}
    </div>
  );
};

export default MainView;