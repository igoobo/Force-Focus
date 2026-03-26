import './TitleBar.css'
import logoIcon from './ForceFocus_icon.png'
import useMainStore from '../../../MainStore.jsx';

// 제목 바 컴포넌트 (상단 바 영역)
export default function TitleBar({ 
  title = 'Force-Focus Web Dashboard', 
  onRefresh, 
  onHelp, 
  onLogout 
}) {
  const setActiveMenu = useMainStore((state) => state.setActiveMenu);
  const openHelp = useMainStore((state) => state.openHelp); // openHelp 가져오기

  const handleLogoClick = () => {
    setActiveMenu('Overview');    // 로고 클릭 시 전역 메뉴 상태를 'Overview'로 변경
  };

  return (
    <header className="titlebar">
      <div className="titlebar__inner">
        {/* 좌측 제목 영역 */}
        <div className="titlebar__left" onClick={handleLogoClick}>
          <div className="titlebar__logo">
            <img src={logoIcon} alt="ForceFocus Logo" className="titlebar__logo-img" />
          </div>
          <div className="titlebar__title">{title}</div>
        </div>

        {/* 우측 메뉴 영역 */}
        <div className="titlebar__right">
          {/* 새로고침 아이콘 버튼 */}
          <button className="titlebar__btn--icon" onClick={onRefresh} data-tooltip="새로고침">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"></path><path d="M21 3v5h-5"></path><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"></path><path d="M8 16H3v5"></path></svg>
          </button>
          
          {/* 도움말 아이콘 버튼 */}
          <button className="titlebar__btn--icon" onClick={openHelp} data-tooltip="도움말">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
          </button>
          
          {/* 로그아웃 버튼 */}
          <button className="titlebar__btn--logout-styled" onClick={onLogout} data-tooltip="로그아웃">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path><polyline points="16 17 21 12 16 7"></polyline><line x1="21" y1="12" x2="9" y2="12"></line></svg>
          </button>
        </div>
      </div>
    </header>
  )
}