// 파일 위치: Force-Focus/desktop-agent/src/components/MainView/MainControls.tsx

import { FC } from 'react';

// Props 인터페이스 정의
interface MainControlsProps {
  sessionStatus: 'active' | 'ended' | null; // 현재 세션의 상태
  onToggleSession: () => void;            // 세션 시작/종료 토글 함수
}

const MainControls: FC<MainControlsProps> = ({ sessionStatus, onToggleSession }) => {
  return (
    <button
      className={`px-10 py-5 rounded-full text-2xl font-bold transition-all duration-300
        ${sessionStatus === 'active' // 세션이 활성 상태일 때
          ? 'bg-red-600 hover:bg-red-700' // 종료 버튼 스타일 (빨간색)
          : 'bg-green-600 hover:bg-green-700' // 시작 버튼 스타일 (초록색)
        }`}
      onClick={onToggleSession} // 버튼 클릭 시 부모로부터 받은 콜백 함수 호출
      aria-label={sessionStatus === 'active' ? '세션 종료' : '세션 시작'} // 스크린 리더를 위한 접근성
    >
      {sessionStatus === 'active' ? '세션 종료' : '세션 시작'} {/* 버튼 텍스트 */}
    </button>
  );
};

export default MainControls;