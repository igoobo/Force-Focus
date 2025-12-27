import React, { useState, useEffect, useRef, useMemo } from "react";
import "./Overview.css";
import useMainStore from "../../../MainStore";
import { useScheduleStore } from "../Schedule/ScheduleStore"; 

// 스케줄 관련 컴포넌트 임포트
import ScheduleDay from "../Schedule/sub/ScheduleDay";
import ScheduleWeek from "../Schedule/sub/ScheduleWeek";
import ScheduleMonth from "../Schedule/sub/ScheduleMonth";

// 활동 요약 데이터 및 로직 임포트
import ActivityChart, { getActivitySummary } from "../ActivitySummary/ActivityChart";

export default function Overview() {
  const setActiveMenu = useMainStore((state) => state.setActiveMenu);
  const schedules = useScheduleStore((state) => state.schedules);
  const [viewMode, setViewMode] = useState("주");
  
  const scrollRef = useRef(null);

  // --- 오늘 날짜 계산 로직 ---
  const today = new Date();
  const dayIndex = today.getDay();
  const year = today.getFullYear();
  const month = today.getMonth() + 1;
  const date = today.getDate();
  const dayOfWeek = ["일", "월", "화", "수", "목", "금", "토"][dayIndex];
  const dateString = `${year}년 ${month}월 ${date}일 (${dayOfWeek})`;

  const dayClassName = dayIndex === 0 ? "sunday" : dayIndex === 6 ? "saturday" : "";
  const summary = useMemo(() => getActivitySummary(), []);

  // 자동 스크롤 로직 (기존 유지)
  useEffect(() => {
    if (viewMode === "월") return;

    const scrollToCurrentTime = () => {
      const now = new Date();
      const currentMinutes = now.getHours() * 60 + now.getMinutes();
      
      let hourHeight = 0;
      if (viewMode === "일") hourHeight = 60;
      if (viewMode === "주") hourHeight = 40;

      if (scrollRef.current && hourHeight > 0) {
        const scrollPosition = (currentMinutes / 60) * hourHeight;
        scrollRef.current.scrollTo({
          top: scrollPosition - 100 > 0 ? scrollPosition - 100 : 0,
          behavior: "smooth"
        });
      }
    };

    const timer = setTimeout(scrollToCurrentTime, 100);
    return () => clearTimeout(timer);
  }, [viewMode]);

  const renderSchedulePreview = () => {
    switch (viewMode) {
      case "일": return <ScheduleDay schedules={schedules} isPreview={true} />;
      case "주": return <ScheduleWeek schedules={schedules} isPreview={true} />;
      case "월": return <ScheduleMonth schedules={schedules} isPreview={true} />;
      default: return <ScheduleDay schedules={schedules} isPreview={true} />;
    }
  };

  return (
    <div className="overview-container">
      {/* 1. 상단 섹션: 스케줄(좌) + 카드 2개(우) */}
      <div className="overview-top-section">
        
        {/* 좌측: 시간표 영역 */}
        <div className="overview-left">
          <div className="overview-header">
            <h4 
              className="schedule-title-link"
              onClick={() => setActiveMenu("스케줄")}
            >
              시간표 ❯
            </h4>

            <div className={`overview-today-date ${dayClassName}`}>
              {dateString}
            </div>

            <div className="view-buttons">
              <button className={viewMode === "일" ? "active" : ""} onClick={() => setViewMode("일")}>일</button>
              <button className={viewMode === "주" ? "active" : ""} onClick={() => setViewMode("주")}>주</button>
              <button className={viewMode === "월" ? "active" : ""} onClick={() => setViewMode("월")}>월</button>
            </div>
          </div>

          <div className="schedule-area">
            <div className="schedule-preview-content" ref={scrollRef}>
              {renderSchedulePreview()}
            </div>
          </div>
        </div>

        {/* 우측: 요약 카드 영역 (1:1 비율) */}
        <div className="overview-right">
          <div className="card summary-card" onClick={() => setActiveMenu("활동 요약")}>
            <h4>최근 작업</h4>
            <p dangerouslySetInnerHTML={{ __html: summary.summarySentence }} />
          </div>
          
          <div className="card feedback-card" onClick={() => setActiveMenu("피드백")}>
            <h4>최근 작업 피드백</h4>
            <p>작업 효율성이 개선되었습니다. 집중 시간이 지난주 대비 안정적입니다.</p>
          </div>
        </div>
      </div>

      {/* 2. 하단 섹션: 그래프 단독 배치 */}
      <div className="overview-bottom-section">
        <div className="card graph-full-card" onClick={() => setActiveMenu("활동 요약")}>
          <h4>최근 작업 그래프</h4>
          <div className="overview-graph-wrapper">
            <ActivityChart /> 
          </div>
        </div>
      </div>
    </div>
  );
}