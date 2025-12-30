import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';
import "./App.css";

import LoginView from './components/LoginView.tsx';
import MainView from './components/MainView';
import { useInterventionListener } from './hooks/useInterventionListener.ts'

function App() {
  // 1. 개입 리스너 훅 (OS 알림 등 처리)
  const { backendError } = useInterventionListener();

  // 2. 상태 관리
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  
  // [초기화] 앱 실행 시 토큰 확인 (자동 로그인 시도)
  useEffect(() => {
    const checkLoginStatus = async () => {
      try {
        // [임시] Rust 백엔드의 토큰 확인 로직
        // 현재는 명시적인 'check_auth' 커맨드가 없으므로, 
        // 일단 로그인 화면으로 시작하도록 설정합니다.
        // 추후: const tokenExists = await invoke('check_auth_status');
        setIsLoggedIn(false); 
      } catch (e) {
        console.error("Auto-login failed:", e);
        setIsLoggedIn(false);
      } finally {
        setIsLoading(false);
      }
    };

    checkLoginStatus();
  }, []);


  // [핸들러] 로그인 성공 시 호출됨 (LoginView에서 호출)
  const handleLoginSuccess = () => {
    console.log("App: Login Success! Switching to MainView.");
    setIsLoggedIn(true);
  };

  
  // [핸들러] 로그아웃 시 호출됨 (MainView에서 호출)
  const handleLogout = async () => {
    try {
      // Rust 백엔드 로그아웃 커맨드 호출 (LSN 토큰 삭제)
      await invoke('logout'); 
      console.log("App: User logged out from backend.");
      setIsLoggedIn(false);
    } catch (e) {
      console.error("Logout failed:", e);
      // 에러가 나더라도 UI상으로는 로그아웃 처리하여 갇히지 않게 함
      setIsLoggedIn(false);
    }
  };

  // 로딩 화면
  if (isLoading) {
    return (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        backgroundColor: '#111827',
        color: 'white',
        fontFamily: 'sans-serif'
      }}>
        Loading...
      </div>
    );
  }


  
  return (
    <div className="App">
      
      {/* 훅에서 반환된 에러 상태를 렌더링 (OS 알림 실패 등) */}
      {backendError && (
        <div style={{ 
          backgroundColor: 'red', color: 'white', 
          padding: '10px', textAlign: 'center',
          position: 'absolute', top: 0, left: 0, right: 0, zIndex: 100
        }}>
          {backendError}
        </div>
      )}

      {isLoggedIn ? (
        <MainView onLogout={handleLogout} />
      ) : (
        <LoginView onLoginSuccess={handleLoginSuccess} />
      )}

    </div>
  );
}

export default App;
