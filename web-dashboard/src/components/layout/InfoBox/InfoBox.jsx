import './InfoBox.css'
import useMainStore from '../../../MainStore.jsx'

// 각 메뉴별 컴포넌트를 불러옴
import Overview from "../../menu/Overview/Overview.jsx" // 메뉴 1 : Overview
import Schedule from "../../menu/Schedule/Schedule.jsx" // 메뉴 2 : 스케줄
import TaskView from "../../menu/Task/TaskView.jsx" // 메뉴 3 : 작업
import ActivitySummary from "../../menu/ActivitySummary/ActivitySummary.jsx" // 메뉴 4 : 활동 요약
import Feedback from "../../menu/Feedback/Feedback.jsx" // 메뉴 5 : 피드백

// InfoBox 컴포넌트 (중앙 정보 표시 컨테이너 영역)
export default function InfoBox() {
  const activeMenu = useMainStore((state) => state.activeMenu);

  const renderComponent = () => {
    switch (activeMenu) {
      case 'Overview':
        return <Overview />;
      case '스케줄':
        return <Schedule />;
      case '작업':
        return <TaskView />;
      case '활동 요약':
        return <ActivitySummary />;
      case '피드백':
        return <Feedback />;
      default:
        return (
          <div className="infobox-content">
            <h2>현재 시스템 상태</h2>
            <p>이 페이지는 “실행 강제 시스템 웹 대시보드”의 상태를 표시합니다.</p>
            <p>왼쪽 메뉴에서 항목을 선택하세요.</p>
          </div>
        );
    }
  };

  return (
    <div className="infobox">
      <div className="infobox-inner">{renderComponent()}</div>
    </div>
  );
}