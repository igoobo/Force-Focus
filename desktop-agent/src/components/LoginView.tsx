import React from 'react';
// 분리된 스타일 파일 import
import { styles } from './LoginView.styles';

import { open } from '@tauri-apps/plugin-shell';


interface LoginViewProps {
  onLoginSuccess: () => void;
}

const LoginView: React.FC<LoginViewProps> = ({ onLoginSuccess }) => {
  // --- 핸들러 로직 ---
  const handleGoogleLogin = async () => {
    try {
      // 환경 변수(.env)에서 API 주소 로드
      const apiBaseUrl = import.meta.env.VITE_API_BASE_URL || 'http://127.0.0.1:8000/api/v1';
      const targetUrl = `${apiBaseUrl}/auth/google/login`;

      console.log(`Opening Google Login page: ${targetUrl}`);
      
      // 시스템 브라우저로 열기 (임시 함수 호출)
      await open(targetUrl);
      
    } catch (e) {
      console.error("Failed to open browser:", e);
    }
  };

  const handleOfflineLogin = () => {
    console.log("Offline Mode Selected - Entering MainView");
    onLoginSuccess();
  };

  // --- 렌더링 ---
  return (
    <>
    <style>
        {`
          html, body, #root {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden !important; /* 스크롤바 강제 제거 */
            background-color: #111827; /* 빈 공간 배경색 일치 */
          }
        `}
      </style>
      
    <div style={styles.container}>
      <div style={styles.header}>
        <h1 style={styles.title}>Force-Focus</h1>
        <p style={styles.subtitle}>몰입을 위한 당신만의 데스크톱 에이전트</p>
      </div>
      
      <div style={styles.buttonContainer}>
        {/* 구글 로그인 버튼 */}
        <button
          onClick={handleGoogleLogin}
          style={styles.googleButton}
          onMouseOver={(e) => e.currentTarget.style.backgroundColor = '#f3f4f6'}
          onMouseOut={(e) => e.currentTarget.style.backgroundColor = 'white'}
          onMouseDown={(e) => e.currentTarget.style.transform = 'scale(0.98)'}
          onMouseUp={(e) => e.currentTarget.style.transform = 'scale(1)'}
        >
          <GoogleIcon style={styles.googleIcon} />
          Google 계정으로 로그인
        </button>

        {/* 구분선 */}
        <div style={styles.dividerContainer}>
          <div style={styles.dividerLine}></div>
          <span style={styles.dividerText}>or</span>
          <div style={styles.dividerLine}></div>
        </div>

        {/* 오프라인 버튼 */}
        <button
          onClick={handleOfflineLogin}
          style={styles.offlineButton}
          onMouseOver={(e) => e.currentTarget.style.backgroundColor = 'rgba(55, 65, 81, 1)'}
          onMouseOut={(e) => e.currentTarget.style.backgroundColor = 'rgba(55, 65, 81, 0.5)'}
          onMouseDown={(e) => e.currentTarget.style.transform = 'scale(0.98)'}
          onMouseUp={(e) => e.currentTarget.style.transform = 'scale(1)'}
        >
          오프라인으로 시작하기
        </button>
      </div>

      <div style={styles.footer}>
        <p style={styles.footerText}>
          로그인 시 데이터 동기화 및 고급 분석 기능이 활성화됩니다.
        </p>
      </div>
    </div>
    </>
  );
};

// --- 서브 컴포넌트: 구글 아이콘 ---
const GoogleIcon: React.FC<{ style?: React.CSSProperties }> = ({ style }) => (
  <svg style={style} viewBox="0 0 24 24">
    <path
      fill="#4285F4"
      d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
    />
    <path
      fill="#34A853"
      d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
    />
    <path
      fill="#FBBC05"
      d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
    />
    <path
      fill="#EA4335"
      d="M12 4.6c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 1.09 14.97 0 12 0 7.7 0 3.99 2.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
    />
  </svg>
);

export default LoginView;