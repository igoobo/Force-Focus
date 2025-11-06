import React from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * '딴짓' 감지 시 *별도의 창*에 표시되는 개입 오버레이 UI
 */
const InterventionOverlay: React.FC = () => {

  const handleFeedbackClick = async () => {
    try {
      console.log("Feedback button clicked. Submitting feedback...");
      await invoke('submit_feedback', { feedbackType: 'is_work' });
      console.log("Feedback submitted successfully.");
    } catch (error) {
      console.error("Failed to submit feedback:", error);
    } finally {
      // Rust의 'hide_overlay' 커맨드를 호출
      console.log("Invoking 'hide_overlay'...");
      try {
        await invoke('hide_overlay');
      } catch (e) {
        console.error("Failed to hide overlay:", e);
      }
    }
  };

  const handleCloseClick = async () => {
    // Rust의 'hide_overlay' 커맨드를 호출
    console.log("Close button clicked. Invoking 'hide_overlay'...");
    try {
      await invoke('hide_overlay');
    } catch (e) {
      console.error("Failed to hide overlay:", e);
    }
  };

  return (
    
    <div 
      style={{
        position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh',
        backgroundColor: 'rgba(0, 0, 0, 0.75)', display: 'flex',
        justifyContent: 'center', alignItems: 'center', zIndex: 9999,
        color: 'white', fontFamily: 'sans-serif',
      }}
    >
      <div 
        style={{
          backgroundColor: '#282c34', padding: '40px', borderRadius: '12px',
          textAlign: 'center', boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
          border: '1px solid #444',
        }}
      >
        <h2 style={{ marginTop: 0, fontSize: '24px' }}>집중할 시간입니다!</h2>
        <p style={{ fontSize: '16px', marginBottom: '30px' }}>
          현재 작업이 '딴짓'으로 감지되었습니다.
        </p>

        <button onClick={handleFeedbackClick} style={{ /* ... */ }}>
          이건 업무임 (피드백)
        </button>
        <button onClick={handleCloseClick} style={{ /* ... */ }}>
          닫기
        </button>
      </div>
    </div>
  );
};

export default InterventionOverlay;