import React from 'react';

interface InterventionOverlayProps {
  /** "이건 업무임" 버튼을 클릭했을 때 호출될 함수 */
  onFeedback: () => void;
  
  /** 오버레이를 닫을 때 호출될 함수 (예: 닫기 버튼) */
  onClose: () => void;
}

/**
 * '딴짓' 감지 시 표시되는 개입 오버레이 UI 컴포넌트
 */
const InterventionOverlay: React.FC<InterventionOverlayProps> = ({ onFeedback, onClose }) => {
  
  const handleFeedbackClick = () => {
    onFeedback();
  };

  const handleCloseClick = () => {
    onClose();
  };

  return (
    <div 
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100vw',
        height: '100vh',
        backgroundColor: 'rgba(0, 0, 0, 0.75)', // 반투명 검은색 배경
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        zIndex: 9999, // 최상단에 표시
        color: 'white',
        fontFamily: 'sans-serif',
      }}
    >
      <div 
        style={{
          backgroundColor: '#282c34', // 어두운 배경색
          padding: '40px',
          borderRadius: '12px',
          textAlign: 'center',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
          border: '1px solid #444',
        }}
      >
        <h2 style={{ marginTop: 0, fontSize: '24px' }}>집중할 시간입니다!</h2>
        <p style={{ fontSize: '16px', marginBottom: '30px' }}>
          현재 작업이 '딴짓'으로 감지되었습니다.
        </p>

        {/* 핵심: 피드백 버튼 */}
        <button
          onClick={handleFeedbackClick}
          style={{
            backgroundColor: '#007bff',
            color: 'white',
            border: 'none',
            padding: '12px 24px',
            fontSize: '16px',
            borderRadius: '8px',
            cursor: 'pointer',
            marginRight: '15px',
            fontWeight: 'bold',
          }}
        >
          이건 업무임 (피드백)
        </button>

        {/* 닫기 버튼 */}
        <button
          onClick={handleCloseClick}
           style={{
            backgroundColor: '#6c757d',
            color: 'white',
            border: 'none',
            padding: '12px 24px',
            fontSize: '16px',
            borderRadius: '8px',
            cursor: 'pointer',
          }}
        >
          닫기
        </button>
      </div>
    </div>
  );
};

export default InterventionOverlay;