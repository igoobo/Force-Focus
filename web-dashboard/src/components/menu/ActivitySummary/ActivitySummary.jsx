import React, { useEffect } from "react";
import "./ActivitySummary.css";
import ActivityChart from "./ActivityChart";
import useMainStore from "../../../MainStore";
import { useActivityStore } from "./ActivityStore";

export default function ActivitySummary() {
  const activityViewMode = useMainStore((state) => state.activityViewMode);
  const setActivityViewMode = useMainStore((state) => state.setActivityViewMode);
  const isDarkMode = useMainStore((state) => state.isDarkMode);
  const { stats, loading, fetchAndAnalyze } = useActivityStore();

  useEffect(() => {
    const CACHE_KEY = "last_activity_fetch_time";
    const ONE_HOUR = 60 * 60 * 1000; // 1시간 (밀리초)
    
    const lastFetch = sessionStorage.getItem(CACHE_KEY);
    const now = Date.now();

    if (lastFetch && (now - parseInt(lastFetch)) < ONE_HOUR && stats.chartData.length > 0) {
      console.log("최근 1시간 이내 기록이 있어 캐시된 데이터를 유지합니다.");
      return;
    }

    fetchAndAnalyze().then(() => {
      sessionStorage.setItem(CACHE_KEY, now.toString());
    });
  }, [fetchAndAnalyze, stats.chartData.length]);

  const toggleLayout = () => {
    const nextMode = activityViewMode === "horizontal" ? "vertical" : "horizontal";
    setActivityViewMode(nextMode);
  };

  if (loading) {
    return (
      <div className={`activity-summary ${activityViewMode} ${isDarkMode ? "dark-theme" : ""}`}>
        <div className="summary-content" style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
          <div className="loader"></div>
          <p style={{ marginLeft: '15px', color: 'var(--text-muted)', fontWeight: '500' }}>
            활동 데이터를 불러오고 있습니다...
          </p>
        </div>
      </div>
    );
  }

  const { summary, chartData } = stats;

  const hasNoData = !chartData || chartData.length === 0 || summary.mainApp === "데이터 없음";

  return (
    <div className={`activity-summary ${activityViewMode} ${isDarkMode ? "dark-theme" : ""}`}>
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
          {hasNoData ? (
            <div className="report-description empty">
              <p>아직 활동 데이터가 존재하지 않습니다. 지금 바로 세션을 시작해 보세요!</p>
            </div>
          ) : (
            <>
              <div className="report-list">
                <ReportItem label="가장 활발한 요일" value={`${summary.busiestDay}요일`} />
                <ReportItem label="주요 사용 앱" value={summary.mainApp} />
                <ReportItem label="평균 집중 시간" value={summary.avgFocusTime} />
                <ReportItem label="전체 집중 강도" value={summary.intensityLevel} highlight />
              </div>
              <div className="report-description">
                <p dangerouslySetInnerHTML={{ __html: summary.summarySentence }} />
              </div>
            </>
          )}
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