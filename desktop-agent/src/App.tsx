import { useState } from "react";
import "./App.css";

import LoginView from './components/LoginView.tsx';
import MainView from './components/MainView';
import { useInterventionListener } from './hooks/useInterventionListener.ts'

function App() {

  const { backendError } = useInterventionListener();

  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);
  
  const handleLoginSuccess = (): void => {
    setIsLoggedIn(true);
    console.log("Mock Login Success! Navigating to Main View.");
  };

  const handleLogout = (): void => {
    setIsLoggedIn(false);
    console.log("Mock Logout. Navigating to Login View.");
  };

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
