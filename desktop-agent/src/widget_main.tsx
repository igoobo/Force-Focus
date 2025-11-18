import React, { useState, useEffect, useCallback } from 'react';
import ReactDOM from 'react-dom/client';
// [수정] v2 API: 'core'(invoke) 및 'event'(listen) import
import { core } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';

// [유지] Rust ActiveSessionInfo (사용되지 않음)
// interface ActiveSessionInfo { ... }

/**
 * [개선] Task 4.12: 글로벌 타이머 위젯 UI
 * [수정] Rust(app_core)의 "widget-tick" 이벤트를 'PUSH' 수신
 */
const WidgetApp: React.FC = () => {
  const [elapsedTime, setElapsedTime] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);

  // [수정] Task 4.12 (P1): Rust(app_core)로부터 1초마다 '틱' 이벤트를 PUSH 수신
  useEffect(() => {
    let unlistenTick: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        // 1. "widget-tick" 이벤트 (1초마다 Rust가 보냄)
        // [!] (ACL 오류 해결) tauri.conf.json의 'widget' 창에 'event-listen' 권한 필요
        const unlistenTickFn = await listen<number>("widget-tick", (e) => {
          setElapsedTime(e.payload); // Rust가 보낸 경과 시간(u64)으로 상태 업데이트
          setError(null);
        });
        unlistenTick = unlistenTickFn;

      } catch (e: any) {
        console.error("Widget listener setup failed:", e);
        setError(e.toString());
      }
    };

    setupListeners();

    // [삭제] PULL 로직 (get_current_session_info)

    return () => {
      if (unlistenTick) unlistenTick();
    };
  }, []); // 마운트 시 1회 실행


  // [삭제] React의 타이머 로직 (setInterval) 완전 삭제

  // [유지] 3. 세션 종료 커맨드 (로직 동일)
  const handleEndSession = useCallback(async () => {
    setError(null);
    try {
      await core.invoke('end_session', { userEvaluationScore: 0 }); 
      setElapsedTime(0); // 즉시 0초로 리셋
    } catch (e: any) {
      setError(e.toString());
    }
  }, []);

  // [유지] 4. 헬퍼 함수
  const formatTime = (seconds: number): string => {
    const h = Math.floor(seconds / 3600).toString().padStart(2, '0');
    const m = Math.floor((seconds % 3600) / 60).toString().padStart(2, '0');
    const s = (seconds % 60).toString().padStart(2, '0');
    return `${h}:${m}:${s}`; 
  };

  // [유지] 5. 렌더링
  return (
    // [!] data-tauri-drag-region은 부모(widget.html)의 body/root에 있음
    <div className="flex items-center justify-between w-full h-full p-3 font-sans text-white select-none">
      {error && <span className="text-red-500 text-xs">{error}</span>}
      
      {elapsedTime > 0 ? (
        <>
          <div data-tauri-drag-region className="flex-grow flex items-center">
            <span className="text-2xl font-mono font-bold text-green-400">
              {formatTime(elapsedTime)}
            </span>
          </div>
          
          <button
            onClick={handleEndSession}
            title="세션 종료"
            className="bg-red-600 hover:bg-red-700 text-white font-bold w-10 h-10 rounded-full flex items-center justify-center transition-transform transform hover:scale-110"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 4h16v16H4z" />
            </svg>
          </button>
        </>
      ) : (
        <div data-tauri-drag-region className="flex-grow flex items-center justify-center">
          <span className="text-gray-400">비활성</span>
        </div>
      )}
    </div>
  );
};

// [유지] React 18 렌더링
ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <WidgetApp />
  </React.StrictMode>
);