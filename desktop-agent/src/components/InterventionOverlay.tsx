import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event'; // [필수] 이벤트 리스너

const InterventionOverlay: React.FC = () => {
  // 현재 모드 상태 관리 ('hidden'은 부모 차원에서 처리되므로 여기선 notification/blocking만)
  const [mode, setMode] = useState<'notification' | 'blocking'>('notification');

  useEffect(() => {
    // Rust 백엔드(app_core.rs)에서 보내는 'intervention-trigger' 이벤트 수신
    const unlistenPromise = listen<string>('intervention-trigger', (event) => {
      console.log("Overlay Event Received:", event.payload);
      
      if (event.payload === 'notification') {
        setMode('notification'); // 경고 모드 (투명)
      } else if (event.payload === 'overlay') {
        setMode('blocking');     // 차단 모드 (불투명)
      }
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

  // 오탐지 신고 (False Positive)
  const handleFeedbackClick = async () => {
    try {
      console.log("Reporting False Positive (Inlier)...");
      
      // Rust 함수의 인자 이름(feedback_type)에 맞춰서 호출
      // 값이 String을 기대하므로 "is_work" 문자열 전달
      await invoke('submit_feedback', { 
        feedbackType: "is_work" 
      });
      
    } catch (error) {
      console.error('Feedback submission failed:', error);
    } finally {
      // [Fail-Safe] 성공하든 실패하든 오버레이는 닫는다.
      console.log("Hiding overlay...");
      await invoke('hide_overlay').catch(err => console.error("Hide failed:", err));
    }
  };

  // 업무 복귀 (True Positive)
  const handleCloseClick = async () => {
    try {
      console.log("Resuming Work...");
      // "work_resumed" 또는 "distraction_acknowledged" 등 적절한 타입 전달
      await invoke('submit_feedback', { 
        feedbackType: "distraction_ignored" 
      });
    } catch (error) {
      console.error('Resume logic failed:', error);
    } finally {
      // 무조건 숨김
      await invoke('hide_overlay').catch(err => console.error("Hide failed:", err));
    }
  };

  // [UI 분기 1] Notification 모드 (경고 단계)
  // 마우스 클릭은 Rust가 OS 레벨에서 통과시키므로, 여기선 시각적 효과만 주면 됨
  if (mode === 'notification') {
    return (
      <div style={{
        position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh',
        border: '8px solid rgba(255, 69, 58, 0.7)', // 붉은색 테두리
        boxSizing: 'border-box',
        pointerEvents: 'none', // [React] 2중 안전장치: DOM 레벨에서도 클릭 통과
        zIndex: 9999,
        display: 'flex', justifyContent: 'center', alignItems: 'flex-start'
      }}>
        {/* 상단에 작게 경고 문구 표시 */}
        <div style={{
          marginTop: '20px',
          padding: '8px 16px',
          backgroundColor: 'rgba(255, 69, 58, 0.9)',
          color: 'white', borderRadius: '20px', fontWeight: 'bold',
          fontSize: '14px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.3)'
        }}>
          ⚠️ 집중력이 흐트러지고 있습니다
        </div>
      </div>
    );
  }

  // [UI 분기 2] Blocking 모드 (차단 단계)
  // 클릭을 받아야 하므로 pointerEvents: 'auto' (기본값)
  return (
    <div 
      style={{
        position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh',
        backgroundColor: 'rgba(0, 0, 0, 0.85)', // 어두운 배경
        display: 'flex', justifyContent: 'center', alignItems: 'center', 
        zIndex: 9999, color: 'white', fontFamily: 'sans-serif',
      }}
    >
      <div style={{
          backgroundColor: '#282c34', padding: '40px', borderRadius: '12px',
          textAlign: 'center', boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
          border: '1px solid #444', minWidth: '300px'
      }}>
        <h2 style={{ marginTop: 0, fontSize: '24px', color: '#ff6b6b' }}>🚫 집중력이 차단되었습니다</h2>
        <p style={{ fontSize: '16px', marginBottom: '30px', color: '#ccc' }}>
          ML 모델이 강한 이탈 패턴을 감지했습니다.<br/>
          지금 하던 행동이 업무와 관련이 있나요?
        </p>

        <div style={{ display: 'flex', gap: '12px', justifyContent: 'center' }}>
            <button onClick={handleFeedbackClick} style={{ 
              padding: '12px 20px', cursor: 'pointer', borderRadius: '6px', border: 'none',
              backgroundColor: '#4a4a4a', color: 'white', fontSize: '14px'
            }}>
              아니요, 업무 중입니다 (오류 신고)
            </button>
            <button onClick={handleCloseClick} style={{ 
              padding: '12px 20px', cursor: 'pointer', borderRadius: '6px', border: 'none',
              backgroundColor: '#ff6b6b', color: 'white', fontWeight: 'bold', fontSize: '14px'
            }}>
              업무 복귀하기
            </button>
        </div>
      </div>
    </div>
  );
};

export default InterventionOverlay;