import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import "./App.css";

import LoginView from './components/LoginView.tsx';
import MainView from './components/MainView';
import { useInterventionListener } from './hooks/useInterventionListener.ts'

type AppView = 'login' | 'main';

function App() {
  // 1. 개입 리스너 훅 (OS 알림 등 처리)
  const { backendError } = useInterventionListener();

  // 2. 상태 관리
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);
  const [userEmail, setUserEmail] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  
  // [초기화] 앱 실행 시 토큰 확인 (기존 유지)
  useEffect(() => {
    const checkLoginStatus = async () => {
      try {
        const email = await invoke<string | null>('check_auth_status');
        if (email) {
          console.log("Auto-login success:", email);
          setUserEmail(email);
          setIsLoggedIn(true);
        } else {
          setIsLoggedIn(false); 
        }
      } catch (e) {
        console.error("Auto-login check failed:", e);
        setIsLoggedIn(false);
      } finally {
        setIsLoading(false);
      }
    };
    checkLoginStatus();
  }, []);


  // 실시간 로그인 이벤트 리스너 
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupAuthListener = async () => {
      try {
        unlisten = await listen<string>('login-success', (event) => {
          console.log(`[App] Real-time Login Success: ${event.payload}`);
          // Rust가 신호를 보내면 즉시 상태 업데이트 -> 화면 전환
          setUserEmail(event.payload);
          setIsLoggedIn(true);
        });
      } catch (e) {
        console.error("Failed to setup auth listener:", e);
      }
    };

    setupAuthListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // --- 핸들러 ---

  // 1. 온라인 로그인 성공 (구글)
  const handleLoginSuccess = (email: string) => {
    console.log(`App: Online Login Success! User: ${email}`);
    setUserEmail(email); // 이메일 저장
    setIsLoggedIn(true);
  };

  // 2. 오프라인 모드 진입
  const handleOfflineMode = () => {
    console.log("App: Entering Offline Mode.");
    setUserEmail(null); // 이메일을 null로 설정해야 'Offline'으로 표시됨
    setIsLoggedIn(true);
  };

  // 3. 로그아웃 시 호출됨 (MainView에서 호출)
  const handleLogout = async () => {
    try {
      // Rust 백엔드 로그아웃 커맨드 호출 (LSN 토큰 삭제)
      await invoke('logout'); 
      console.log("App: User logged out from backend.");
      setUserEmail(null); // 이메일 초기화
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
        <MainView onLogout={handleLogout} userEmail={userEmail} />
      ) : (
        <LoginView 
          onLoginSuccess={handleLoginSuccess} 
          onOfflineClick={handleOfflineMode} 
        />
      )}

    </div>
  );
}

export default App;
