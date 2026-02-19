// src/components/Activity/ActivitySummary.jsx
import React, { useEffect } from "react";
import "./ActivitySummary.css";
import ActivityChart from "./ActivityChart";
import useMainStore from "../../../MainStore";
import { useActivityStore } from "./ActivityStore";

export default function ActivitySummary() {
  const activityViewMode = useMainStore((state) => state.activityViewMode);
  const setActivityViewMode = useMainStore((state) => state.setActivityViewMode);
  const { stats, loading, fetchAndAnalyze } = useActivityStore();

  useEffect(() => {
    fetchAndAnalyze();
  }, [fetchAndAnalyze]);

  const toggleLayout = () => {
    const nextMode = activityViewMode === "horizontal" ? "vertical" : "horizontal";
    setActivityViewMode(nextMode);
  };

  if (loading) {
    return (
      <div className={`activity-summary ${activityViewMode}`}>
        <div className="summary-header">
          <span className="summary-title">📊 주간 활동 요약 리포트</span>
        </div>
        <div className="summary-content" style={{ justifyContent: 'center', alignItems: 'center' }}>
          <p>활동 데이터를 분석 중입니다...</p>
        </div>
      </div>
    );
  }

  const { summary, chartData } = stats;

  return (
    <div className={`activity-summary ${activityViewMode}`}>
      <div className="summary-header">
        <span className="summary-title">📊 주간 활동 요약 리포트</span>
        <button onClick={toggleLayout} className="toggle-btn">
          {activityViewMode === "vertical" ? "가로로 보기" : "세로로 보기"}
        </button>
      </div>

      <div className="summary-content">
        <div className="summary-graph">
          <h3>일별 활동 및 집중 강도</h3>
          <div className="graph-placeholder">
            <ActivityChart data={chartData} />
          </div>
        </div>

        <div className="summary-report">
          <h3>활동 분석 요약 보고서</h3>
          <div className="report-list">
            <ReportItem label="가장 활발한 요일" value={`${summary.busiestDay}요일`} />
            <ReportItem label="주요 사용 앱" value={summary.mainApp} />
            <ReportItem label="평균 집중 시간" value={summary.avgFocusTime} />
            <ReportItem label="전체 집중 강도" value={summary.intensityLevel} highlight />
          </div>
          <div className="report-description">
            <p dangerouslySetInnerHTML={{ __html: summary.summarySentence }} />
          </div>
        </div>
      </div>
    </div>
  );
}

// 리포트 개별 항목 컴포넌트
const ReportItem = ({ label, value, highlight }) => (
  <div className="report-item">
    <span className="label">{label}</span>
    <span className={`value ${highlight ? 'highlight' : ''}`}>{value}</span>
  </div>
);