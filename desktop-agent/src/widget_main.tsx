import React, { useState, useEffect, useCallback } from 'react';
import ReactDOM from 'react-dom/client';
// [수정] v2 API: 'core'(invoke) 및 'event'(listen) import
import { core } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import './App.css';
// [유지] Rust ActiveSessionInfo (PULL 타입)
interface ActiveSessionInfo {
  session_id: string;
  task_id: string | null;
  start_time_s: number; 
}

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
    // [개선] 'Frosted Glass' (반투명, 블러), 둥근 모서리, 그림자
    <div 
  style={{ 
    // [컨테이너] 흰색 배경, 둥근 모서리, 부드러운 그림자
    width: '100%',
    height: '100%',
    backgroundColor: '#ffffff', 
    borderRadius: '12px', 
    boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '0 16px',
    boxSizing: 'border-box',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    overflow: 'hidden', // 둥근 모서리 밖으로 내용이 나가는 것 방지
    // [상태 표시] 세션 활성 여부에 따라 왼쪽 테두리 색상 변경 (세련된 인디케이터 역할)
    borderLeft: elapsedTime !== 0 ? '6px solid #10B981' : '6px solid #CBD5E1' 
  }}
>
  {/* 에러 메시지 영역 */}
  {error && (
    <div 
      data-tauri-drag-region 
      style={{ flexGrow: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}
    >
      <span style={{ color: '#EF4444', fontSize: '13px', fontWeight: '500' }} title={error}>
        {error}
      </span>
    </div>
  )}
  
  {/* --- 활성 상태 (타이머 + 종료 버튼) --- */}
  {elapsedTime !== 0 && !error && (
    <>
      {/* 타이머 영역 (드래그 가능) */}
      <div 
        data-tauri-drag-region 
        style={{ 
          flexGrow: 1, 
          display: 'flex', 
          flexDirection: 'column', // 상하 배치로 변경하여 "FOCUS" 라벨 추가 가능성 열어둠
          justifyContent: 'center'
        }}
      >
        <span style={{ 
          fontSize: '11px', 
          color: '#10B981', // 에메랄드 색상
          fontWeight: 'bold', 
          letterSpacing: '1px',
          marginBottom: '-2px',
          textTransform: 'uppercase'
        }}>
          FOCUSING
        </span>
        <span style={{ 
          fontSize: '28px', 
          fontWeight: '700', 
          color: '#1F2937', // 진한 회색 (완전 검정보다 고급스러움)
          fontFamily: '"SF Mono", "Menlo", "Monaco", "Courier New", monospace',
          letterSpacing: '-0.5px'
        }}>
          {formatTime(elapsedTime)}
        </span>
      </div>
      
      {/* 종료 버튼 (아이콘 대신 텍스트 유지하되, Pill 형태의 버튼으로 변경) */}
      <button
        onClick={handleEndSession}
        style={{ 
          backgroundColor: '#FEE2E2', // 연한 빨간 배경
          color: '#EF4444', // 진한 빨간 텍스트
          padding: '8px 16px', 
          border: 'none', 
          borderRadius: '9999px', // Pill Shape
          fontWeight: '600',
          cursor: 'pointer',
          fontSize: '13px',
          transition: 'all 0.2s ease',
          outline: 'none',
          marginLeft: '12px'
        }}
        onMouseOver={(e) => {
            e.currentTarget.style.backgroundColor = '#EF4444';
            e.currentTarget.style.color = 'white';
        }}
        onMouseOut={(e) => {
            e.currentTarget.style.backgroundColor = '#FEE2E2';
            e.currentTarget.style.color = '#EF4444';
        }}
      >
        종료
      </button>
    </>
  )}

  {/* --- 비활성 상태 --- */}
  {elapsedTime === 0 && !error && (
    <div 
      data-tauri-drag-region 
      style={{ 
        flexGrow: 1, 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'space-between' 
      }}
    >
      <span style={{ 
        color: '#94A3B8', // 쿨 그레이
        fontSize: '14px', 
        fontWeight: '500' 
      }}>
        세션 대기 중
      </span>
      {/* 비활성 상태일 때 시각적 균형을 위한 장식용 요소 혹은 시작 버튼 */}
      <div style={{ width: '8px', height: '8px', borderRadius: '50%', backgroundColor: '#CBD5E1' }}></div>
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