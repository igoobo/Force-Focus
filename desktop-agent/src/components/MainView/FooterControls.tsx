// 파일 위치: Force-Focus/desktop-agent/src/components/MainView/FooterControls.tsx

import { FC } from 'react';

// Props 인터페이스 정의
interface FooterControlsProps {
  onLogout: () => void; // 로그아웃 버튼 클릭 시 호출될 콜백 함수
}

const FooterControls: FC<FooterControlsProps> = ({ onLogout }) => {
  return (
    <div className="w-full flex justify-end"> 
      <button
        onClick={onLogout}
        className="px-4 py-2 bg-gray-600 hover:bg-gray-700 rounded-md text-sm font-semibold"
        aria-label="로그아웃" // 스크린 리더를 위한 접근성
      >
        로그아웃 (Mock)
      </button>
    </div>
  );
};

export default FooterControls;