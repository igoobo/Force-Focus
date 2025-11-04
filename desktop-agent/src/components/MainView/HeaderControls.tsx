// 파일 위치: Force-Focus/desktop-agent/src/components/MainView/HeaderControls.tsx

import{ FC } from 'react';
import { MdSettings, MdOutlineDashboard, MdSync, MdSyncProblem } from 'react-icons/md';

// Props 인터페이스 정의
interface HeaderControlsProps {
  syncStatus: "online" | "offline" | "syncing";
  onOpenSettings: () => void; // 설정 화면 열기 함수 (미구현)
  onGoToDashboard: () => void; // 대시보드로 이동 함수 (미구현)
}

const HeaderControls: FC<HeaderControlsProps> = ({ syncStatus, onOpenSettings, onGoToDashboard }) => {
  // 동기화 상태에 따라 다른 UI를 렌더링하는 헬퍼 함수
  const renderSyncStatus = () => {
    switch (syncStatus) {
      case "online": return <span className="text-green-400 flex items-center"><MdSync className="mr-1 animate-spin-slow" />Online</span>;
      case "syncing": return <span className="text-yellow-400 flex items-center animate-pulse"><MdSync className="mr-1" />Syncing...</span>;
      case "offline": return <span className="text-red-400 flex items-center"><MdSyncProblem className="mr-1" />Offline</span>;
      default: return null;
    }
  };

  return (
    <div className="w-full flex justify-between items-center">
      {/* 왼쪽 아이콘 섹션 */}
      <div className="flex items-center space-x-4">
        <button
          onClick={onOpenSettings}
          className="text-gray-300 hover:text-white transition-colors"
          aria-label="설정 열기" // 스크린 리더를 위한 접근성
        >
          <MdSettings size={28} /> {/* 설정 아이콘 */}
        </button>
        <button
          onClick={onGoToDashboard}
          className="text-gray-300 hover:text-white transition-colors"
          aria-label="웹 대시보드로 이동" // 스크린 리더를 위한 접근성
        >
          <MdOutlineDashboard size={28} /> {/* 웹 대시보드 아이콘 */}
        </button>
      </div>

      {/* 오른쪽 동기화 상태 섹션 */}
      <div className="text-sm">
        {renderSyncStatus()}
      </div>
    </div>
  );
};

export default HeaderControls;