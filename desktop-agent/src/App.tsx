import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import "./App.css";

import LoginView from './components/LoginView.tsx';
import MainView from './components/MainView';
import SettingsView from './components/SettingsView';
import { useInterventionListener } from './hooks/useInterventionListener.ts'

type AppView = 'login' | 'main' | 'settings';

function App() {
  // 1. 개입 리스너 훅 (OS 알림 등 처리)
  const { backendError } = useInterventionListener();

  // 2. 상태 관리
  const [currentView, setCurrentView] = useState<AppView>('login');
  const [userEmail, setUserEmail] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  
  // 1. [초기화] 앱 실행 시 LSN 토큰 확인 (자동 로그인)
  useEffect(() => {
    const checkLoginStatus = async () => {
      try {
        const email = await invoke<string | null>('check_auth_status');
        if (email) {
          console.log("Auto-login success:", email);
          setUserEmail(email);
          setCurrentView('main');
        } else {
          setCurrentView('login');
        }
      } catch (e) {
        console.error("Auto-login check failed:", e);
        setCurrentView('login');
      } finally {
        setIsLoading(false);
      }
    };
    checkLoginStatus();
  }, []);


  // 2. [이벤트] 실시간 로그인 성공 감지
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    const setupAuthListener = async () => {
      try {
        unlisten = await listen<string>('login-success', (event) => {
          console.log(`[App] Login Event Received! User: ${event.payload}`);
          setUserEmail(event.payload);
          setCurrentView('main');
        });
      } catch (e) {
        console.error("Failed to setup auth listener:", e);
      }
    };
    setupAuthListener();
    return () => { if (unlisten) unlisten(); };
  }, []);

  // --- 핸들러 ---

  // 온라인 로그인 성공
  const handleLoginSuccess = (email: string) => {
    console.log(`App: Online Login Success! User: ${email}`);
    setUserEmail(email);
    setCurrentView('main');
  };

  // 오프라인 모드 진입
  const handleOfflineMode = () => {
    console.log("App: Entering Offline Mode.");
    setUserEmail(null);
    setCurrentView('main');
  };

  // 설정 화면 열기
  const handleOpenSettings = () => {
    setCurrentView('settings');
  };

  // 설정에서 뒤로가기
  const handleBackToMain = () => {
    setCurrentView('main');
  };

  // 로그아웃
  const handleLogout = async () => {
    try {
      if (userEmail) {
        await invoke('logout');
        console.log("App: User logged out.");
      }
      setUserEmail(null); 
      setCurrentView('login');
    } catch (e) {
      console.error("Logout failed:", e);
      setCurrentView('login');
    }
  };

  if (isLoading) {
    return (
      <div style={styles.loadingContainer}>Loading...</div>
    );
  }

  // 화면 라우팅
  const renderView = () => {
    switch (currentView) {
      case 'main':
        return (
          <MainView 
            onLogout={handleLogout} 
            onOpenSettings={handleOpenSettings}
            userEmail={userEmail} 
          />
        );
      case 'settings':
        return (
          <SettingsView 
            userEmail={userEmail} 
            onLogout={handleLogout} 
            onBack={handleBackToMain} 
          />
        );
      case 'login':
      default:
        return (
          <LoginView 
            onLoginSuccess={handleLoginSuccess} 
            onOfflineClick={handleOfflineMode} 
          />
        );
    }
  };

  return (
    <div>
      {backendError && <div style={styles.errorBanner}>{backendError}</div>}
      {renderView()}
    </div>
  );
}

const styles: { [key: string]: React.CSSProperties } = {

  loadingContainer: {
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    height: '100vh', backgroundColor: '#111827', color: 'white', fontFamily: 'sans-serif'
  },
  errorBanner: {
    backgroundColor: '#ef4444', color: 'white', padding: '10px', textAlign: 'center',
    position: 'absolute', top: 0, left: 0, right: 0, zIndex: 50,
    fontSize: '14px', fontWeight: 'bold', boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
  }
};

export default App;
