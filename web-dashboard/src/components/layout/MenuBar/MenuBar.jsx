import './MenuBar.css'
import useMainStore from '../../../MainStore.jsx'
import { useState, useEffect, useRef } from 'react'
import axios from 'axios'

export default function MenuBar() {
  const { 
    isOpen, toggleMenu, activeMenu, setActiveMenu, 
    isDarkMode, toggleDarkMode, isDirty, setIsDirty, logout 
  } = useMainStore();

  const [userData, setUserData] = useState({ email: '', lastLogin: '' });
  const [isPopupOpen, setIsPopupOpen] = useState(false);

  const popoverRef = useRef(null);

  const menus = [
    { icon: '🏠', label: 'Overview' },
    { icon: '📝', label: '스케줄' },
    { icon: '🛠️', label: '작업' },
    { icon: '📊', label: '활동 요약' },
    { icon: '🚨', label: '피드백' },
  ]

  useEffect(() => {
    const fetchUserInfo = async () => {
      try {
        const token = localStorage.getItem('accessToken');
        if (token) {
          const response = await axios.get('/users/me', {
            headers: { Authorization: `Bearer ${token}` }
          });

          // 1. 서버에서 가져온 원본 데이터 (예: "2026-01-01 00:00:00")
          let rawDate = response.data.last_login_at;
          let formattedDate = '기록 없음';

          if (rawDate) {
            // 1. DB의 UTC 시간을 자바스크립트가 인식할 수 있게 변환 (Z 추가)
            const utcDateStr = rawDate.endsWith('Z') ? rawDate : `${rawDate.replace(' ', 'T')}Z`;
            const date = new Date(utcDateStr);
            
            // 2. 한국 시간(KST, UTC+9)으로 오프셋을 적용하여 각 항목 추출
            const d = new Intl.DateTimeFormat('ko-KR', {
              timeZone: 'Asia/Seoul',
              year: 'numeric', month: '2-digit', day: '2-digit',
              hour: '2-digit', minute: '2-digit', second: '2-digit',
              hour12: false
            }).formatToParts(date);

            // 3. 추출된 요소들을 p 객체에 매핑
            const p = {};
            d.forEach(({ type, value }) => { p[type] = value; });

            // 4. 최종 문자열 조립: 하이픈(-)과 공백( )을 명시적으로 추가함
            // 결과 예시: 2026-01-01 00:00:00
            formattedDate = `${p.year}-${p.month}-${p.day} ${p.hour}:${p.minute}:${p.second}`;
          }
          setUserData({
            email: response.data.email,
            lastLogin: formattedDate
          });
        }
      } catch (err) {
        console.error("사용자 정보 조회 실패:", err);
      }
    };

    fetchUserInfo();
  }, []);

  // 3. 외부 클릭 감지 로직
  useEffect(() => {
    function handleClickOutside(event) {
      if (isPopupOpen && 
          popoverRef.current && 
          !popoverRef.current.contains(event.target) &&
          !event.target.closest('.user-profile-item')) {
        setIsPopupOpen(false);
      }
    }

    // 마우스 다운 이벤트 리스너 등록
    document.addEventListener("mousedown", handleClickOutside);
    
    // 컴포넌트 언마운트 시 리스너 해제
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isPopupOpen]);

  const handleMenuClick = (menuLabel) => {
    if (activeMenu === '작업' && menuLabel !== '작업' && isDirty) {
      const leaveConfirm = window.confirm("변경된 사항이 저장되지 않았습니다."); 
      if (!leaveConfirm) return;
      setIsDirty(false);
    }
    setActiveMenu(menuLabel);
  };

  const handleLogoutClick = () => {
    if (window.confirm("로그아웃 하시겠습니까?")) {
      logout(); // MainStore의 logout 실행 (토큰 삭제 및 상태 변경)
    }
  };

  const handleDeleteAccount = async () => {
    if (window.confirm("정말로 계정을 삭제하시겠습니까? 계정에 연결된 데이터가 모두 삭제됩니다.")) {
      try {
        const token = localStorage.getItem('accessToken');
        await axios.delete('/users/me', {
          headers: { Authorization: `Bearer ${token}` }
        });
        alert("계정이 삭제되었습니다.");
        localStorage.removeItem('accessToken');
        window.location.href = '/login';
      } catch (err) {
        alert("계정 삭제 중 오류가 발생했습니다.");
      }
    }
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
          <li 
            className="menu-bar__item user-profile-item" 
            onClick={() => setIsPopupOpen(!isPopupOpen)}
          >
            <div className="menu-bar__user-avatar">
              {userData.email 
                ? userData.email.charAt(0).toUpperCase() 
                : '?'} 
            </div>
            <div className="menu-bar__user-info">
              <span className="menu-bar__label">사용자 정보</span>
              <span className="menu-bar__email">{userData.email || "불러오는 중..."}</span>
            </div>

            {isPopupOpen && (
              <div className="user-popover" ref={popoverRef} onClick={(e) => e.stopPropagation()}>
                <button 
                  className="user-popover__close" 
                  onClick={(e) => {
                    e.stopPropagation(); // li 태그의 토글 이벤트 전파 방지
                    setIsPopupOpen(false);
                  }}
                  aria-label="닫기"
                >
                  &times;
                </button>
                
                <div className="user-popover__content">
                    <div className="user-popover__info-card">
                      <strong>이메일</strong>
                      <div className="user-popover__email-row">
                        <div className="menu-bar__user-avatar small">
                          {userData.email ? userData.email.charAt(0).toUpperCase() : '?'}
                        </div>
                        <span>{userData.email}</span>
                      </div>
                    </div>
                    <div className="user-popover__info-card">
                      <strong>마지막 로그인 (KST)</strong>
                      <p>{userData.lastLogin}</p>
                    </div>
                  </div>
                    <div className="user-popover__footer">
                      {/* 로그아웃 버튼 */}
                      <button className="btn-logout" onClick={handleLogoutClick}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                          <polyline points="16 17 21 12 16 7" />
                          <line x1="21" y1="12" x2="9" y2="12" />
                        </svg>
                        {'\u00A0'}{'\u00A0'}로그아웃
                      </button>

                      {/* 계정 삭제 버튼 */}
                      <button className="btn-delete" onClick={handleDeleteAccount}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M3 6h18" />
                          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                          <line x1="10" y1="11" x2="10" y2="17" />
                          <line x1="14" y1="11" x2="14" y2="17" />
                        </svg>
                        {'\u00A0'}{'\u00A0'}계정 삭제
                      </button>
                    </div>
                <div className="user-popover__arrow"></div>
              </div>
            )}
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