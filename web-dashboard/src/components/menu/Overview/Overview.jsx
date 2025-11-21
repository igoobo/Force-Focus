import React from "react";
import "./Overview.css";
import useMainStore from "../../../MainStore";

export default function Overview() {
  const setActiveMenu = useMainStore((state) => state.setActiveMenu);

  return (
    <div className="overview-container">
      {/* 좌측 일정 영역 */}
      <div className="overview-left">
        <div className="overview-header">
          <div className="view-buttons">
            <button>일</button>
            <button>주</button>
            <button>월</button>
          </div>
          <div className="action-buttons">
            <button className="add-btn">추가</button>
            <button className="delete-btn">삭제</button>
          </div>
        </div>

        {/* '시간표' 클릭 → 스케줄 메뉴로 전환 */}
        <div
          className="schedule-area"
          onClick={() => setActiveMenu("스케줄")}
          style={{ cursor: "pointer" }}
        >
          <h4>시간표</h4>
          <div className="schedule-placeholder">
            {/* 캘린더나 표 렌더링 */}
          </div>
        </div>
      </div>

      {/* 우측 정보 영역 */}
      <div className="overview-right">

        {/* 최근 작업 → 활동 요약 */}
        <div
          className="card"
          onClick={() => setActiveMenu("활동 요약")}
          style={{ cursor: "pointer" }}
        >
          <h4>최근 작업</h4>
          <p>최근 작업 내역 표시 예정</p>
        </div>

        {/* 최근 작업 피드백 → 피드백 메뉴 */}
        <div
          className="card"
          onClick={() => setActiveMenu("피드백")}
          style={{ cursor: "pointer" }}
        >
          <h4>최근 작업 피드백</h4>
          <p>피드백 데이터 표시 예정</p>
        </div>

        {/* 최근 작업 그래프 → 활동 요약 */}
        <div
          className="card"
          onClick={() => setActiveMenu("활동 요약")}
          style={{ cursor: "pointer" }}
        >
          <h4>최근 작업 그래프</h4>
          <p>
            그래프 영역 표시 예정 <br />
            (예: Recharts, Chart.js)
          </p>
        </div>
      </div>
    </div>
  );
}