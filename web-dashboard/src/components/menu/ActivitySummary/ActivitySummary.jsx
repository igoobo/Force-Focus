import React, { useState, useMemo } from "react";
import "./ActivitySummary.css";
import ActivityChart, { getActivitySummary } from "./ActivityChart";

export default function ActivitySummary() {
  const [isVertical, setIsVertical] = useState(false);
  const toggleLayout = () => setIsVertical(!isVertical);

  const summary = useMemo(() => getActivitySummary(), []);

  return (
    <div className={`activity-summary ${isVertical ? "vertical" : "horizontal"}`}>
      <div className="summary-header">
        <span className="summary-title">📊 주간 활동 요약 리포트</span>
        <button onClick={toggleLayout} className="toggle-btn">
          {isVertical ? "가로로 보기" : "세로로 보기"}
        </button>
      </div>

      <div className="summary-content">
        <div className="summary-graph">
          <h3>일별 활동 및 집중 강도</h3>
          <div className="graph-placeholder">
            {/* 공통 차트 컴포넌트 사용 */}
            <ActivityChart />
          </div>
        </div>

        <div className="summary-report">
        <h3>활동 분석 요약 보고서</h3>
        <div className="report-list">
          <div className="report-item">
            <span className="label">가장 활발한 요일</span>
            <span className="value">{summary.busiestDay}요일</span>
          </div>
          <div className="report-item">
            <span className="label">주요 사용 앱</span>
            <span className="value">{summary.mainApp}</span>
          </div>
          <div className="report-item">
            <span className="label">평균 집중 시간</span>
            <span className="value">{summary.avgFocusTime}</span>
          </div>
          <div className="report-item">
            <span className="label">전체 집중 강도</span>
            <span className="value highlight">{summary.intensityLevel}</span>
          </div>
        </div>
        <div className="report-description">
          <p dangerouslySetInnerHTML={{ __html: summary.summarySentence }} />
        </div>
      </div>
      </div>
    </div>
  );
}