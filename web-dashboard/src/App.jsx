import { useEffect, useState } from 'react';
import './App.css';
import TitleBar from './components/layout/TitleBar/TitleBar.jsx';
import InfoBox from './components/layout/InfoBox/InfoBox.jsx';
import MenuBar from './components/layout/MenuBar/MenuBar.jsx';
import useMainStore from './MainStore.jsx';
import HelpModal from './components/layout/Help/HelpModal.jsx';
import Login from './components/login/login.jsx';

function App() {
  const { 
    isHelpOpen, openHelp, setActiveMenu, isDarkMode, 
    isLoggedIn, login, logout, activeMenu 
  } = useMainStore();

  // 로그아웃 로직
  const handleLogout = () => {
    if (window.confirm("로그아웃 하시겠습니까?")) {
      logout();
    }
  };

  // activeMenu가 변경될 때마다 스크롤을 맨 위로 이동
  useEffect(() => {
    window.scrollTo(0, 0);

    const mainContent = document.querySelector('.main-content-area');
    if (mainContent) {
      mainContent.scrollTo(0, 0);
    }
  }, [activeMenu]);

  // 새로고침 시(컴포넌트 마운트 시) 무조건 Overview 메뉴로 이동
  useEffect(() => {
    if (isLoggedIn && setActiveMenu) {
      setActiveMenu('Overview');
    }
  }, [isLoggedIn, setActiveMenu]);

  // 스토어상 로그인은 되어있는데 실제 토큰이 없다면 비정상 세션으로 간주
  useEffect(() => {
  const token = localStorage.getItem('accessToken');
  if (isLoggedIn && !token) {
    logout(); 
  }
}, [isLoggedIn, logout]);

  // 로그인하지 않은 경우 로그인 화면 렌더링
  if (!isLoggedIn) {
    return (
      <Login onLoginSuccess={login} />
    );
  }

  return (
    // 다크모드 상태에 따라 클래스 동적 부여
    <div className={`app-root ${isDarkMode ? 'dark-theme' : ''}`}>
      <TitleBar
        onRefresh={() => location.reload()}
        onHelp={openHelp}
        onLogout={handleLogout} 
      />
      
      <MenuBar />
      
      {/* 메인 콘텐츠 영역: 배경색을 CSS 변수(var(--bg-main))로 처리하여 다크모드 연동 */}
      <main
        style={{
          marginLeft: '20%',
          paddingTop: '56px',
          height: 'calc(100vh - 56px)',
          backgroundColor: 'var(--bg-main)', 
          transition: 'background-color 0.3s ease'
        }}
      >
      </main>

     <InfoBox />

      {isHelpOpen && <HelpModal />}
    </div>
  );
}

export default App;