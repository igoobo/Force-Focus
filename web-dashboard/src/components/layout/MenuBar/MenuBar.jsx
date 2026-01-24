import './MenuBar.css'
import useMainStore from '../../../MainStore.jsx'
import { useState, useEffect } from 'react'
import axios from 'axios'

export default function MenuBar() {
  const { 
    isOpen, toggleMenu, activeMenu, setActiveMenu, 
    isDarkMode, toggleDarkMode, isDirty, setIsDirty 
  } = useMainStore();

  const [userEmail, setUserEmail] = useState('');

  const menus = [
    { icon: '🏠', label: 'Overview' },
    { icon: '📝', label: '스케줄' },
    { icon: '🛠️', label: '작업' },
    { icon: '📊', label: '활동 요약' },
    { icon: '🚨', label: '피드백' },
    { icon: '⚙️', label: '설정' },
  ]

  useEffect(() => {
    const getEmail = async () => {
      try {
        const token = localStorage.getItem('accessToken');
        if (token) {
          const response = await axios.get('/api/v1/users', {
            headers: { Authorization: `Bearer ${token}` }
          });
          setUserEmail(response.data.email);
        }
      } catch (err) {
        console.error("이메일 조회 실패:", err);
        setUserEmail("정보를 불러올 수 없음");
      }
    };

    getEmail();
  }, []);

  const handleMenuClick = (menuLabel) => {
    if (activeMenu === '작업' && menuLabel !== '작업' && isDirty) {
      const leaveConfirm = window.confirm("변경된 사항이 저장되지 않았습니다."); 
      if (!leaveConfirm) return;
      setIsDirty(false);
    }
    setActiveMenu(menuLabel); // 메뉴 이동 승인
  };

  return (
    <aside className={`menu-bar ${isOpen ? '' : 'collapsed'} ${isDarkMode ? 'dark-theme' : ''}`}>
      <div className="menu-bar__header">
        <span className="menu-bar__title">{isOpen ? 'MENU' : '≡'}</span>
        <button className="menu-bar__toggle" onClick={toggleMenu}>
          {isOpen ? '←' : '≡'}
        </button>
      </div>

      <nav className="menu-bar__nav">
        <ul className="menu-bar__list">
          {menus.map((menu) => (
            <li
              key={menu.label}
              className={`menu-bar__item ${activeMenu === menu.label ? 'active' : ''}`}
              onClick={() => handleMenuClick(menu.label)}
            >
              <span className="menu-bar__icon">{menu.icon}</span>
              {isOpen && <span className="menu-bar__label">{menu.label}</span>}
            </li>
          ))}
        </ul>
      </nav>

      <div className="menu-bar__footer">
        <ul className="menu-bar__list">
          <li className="menu-bar__item user-profile-item">
            <span className="menu-bar__icon">👤</span>
            <div className="menu-bar__user-info">
              <span className="menu-bar__label">사용자 정보</span>
              <span className="menu-bar__email">{userEmail || "불러오는 중..."}</span>
            </div>
          </li>

          <li className="menu-bar__item theme-toggle-item" onClick={toggleDarkMode}>
            <span className="menu-bar__icon">{isDarkMode ? '☀️' : '🌙'}</span>
            {isOpen && (
              <span className="menu-bar__label">
                {isDarkMode ? '라이트 모드' : '다크 모드'}
              </span>
            )}
          </li>
        </ul>
      </div>
    </aside>
  )
}