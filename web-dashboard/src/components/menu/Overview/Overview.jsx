import React, { useState, useEffect, useRef, useMemo } from "react";
import "./Overview.css";
import useMainStore from "../../../MainStore";
import { useScheduleStore } from "../Schedule/ScheduleStore";
import authApi from "../../../api/authApi";

// 스케줄 관련 컴포넌트 임포트
import ScheduleDay from "../Schedule/sub/ScheduleDay";
import ScheduleWeek from "../Schedule/sub/ScheduleWeek";
import ScheduleMonth from "../Schedule/sub/ScheduleMonth";

// 활동 요약 데이터 및 로직 임포트
import ActivityChart from "../ActivitySummary/ActivityChart";
import { useActivityStore } from "../ActivitySummary/ActivityStore";

export default function Overview() {
  const setActiveMenu = useMainStore((state) => state.setActiveMenu);
  const schedules = useScheduleStore((state) => state.schedules);
  const [viewMode, setViewMode] = useState("주");
  const { fetchSchedules } = useScheduleStore();
  
  const { stats, fetchAndAnalyze, loading } = useActivityStore();

  const [previewDate, setPreviewDate] = useState(new Date());
  const scrollRef = useRef(null);

  // 피드백 데이터 상태 관리 및 전역 캐시 활용
  const feedbackCache = useMainStore((state) => state.feedbackCache);
  const setFeedbackCache = useMainStore((state) => state.setFeedbackCache);
  const [feedbackData, setFeedbackData] = useState(null);
  const [isFeedbackLoading, setIsFeedbackLoading] = useState(true);
  // 가장 최근 세션의 ID를 정확히 저장하기 위한 상태 추가
  const [latestId, setLatestId] = useState("");

  // 최신 세션 피드백 자동 로드 로직 추가
  useEffect(() => {
    const fetchLatestFeedback = async () => {
      setIsFeedbackLoading(true);
      try {
        // 1. 세션 목록에서 가장 최근 항목 1개 가져오기
        const sessionsResponse = await authApi.get("/api/v1/sessions/?limit=1");
        const sessions = Array.isArray(sessionsResponse.data) 
          ? sessionsResponse.data 
          : (sessionsResponse.data.sessions || []);

        if (sessions && sessions.length > 0) {
          const latestSessionId = sessions[0].id;
          
          // 찾은 최신 ID를 상태에 저장
          setLatestId(latestSessionId);

          // 2. 캐시 확인 후 없으면 API 호출
          if (feedbackCache[latestSessionId]) {
            setFeedbackData(feedbackCache[latestSessionId]);
          } else {
            const feedbackResponse = await authApi.get(`/api/v1/insight/analyze/${latestSessionId}`);
            const freshData = feedbackResponse.data;
            setFeedbackData(freshData);
            
            // 전역 스토어 캐시에 저장
            setFeedbackCache({ ...feedbackCache, [latestSessionId]: freshData });
          }
          
          // [수정] 피드백 메뉴 진입 시 세션 선택을 건너뛰도록 ID 설정하는 로직을 여기서 제거합니다.
          // (마운트 시 자동 실행 방지)
        }
      } catch (err) {
        console.error("최근 피드백 로드 실패:", err);
      } finally {
        setIsFeedbackLoading(false);
      }
    };

    fetchLatestFeedback();
  }, []);

  // --- 오늘 날짜 계산 로직 ---
  const today = new Date();
  const dayIndex = today.getDay();
  const year = today.getFullYear();
  const month = today.getMonth() + 1;
  const date = today.getDate();
  const dayOfWeek = ["일", "월", "화", "수", "목", "금", "토"][dayIndex];
  const dateString = `${year}년 ${month}월 ${date}일 (${dayOfWeek})`;

  const dayClassName = dayIndex === 0 ? "sunday" : dayIndex === 6 ? "saturday" : "";
  const summary = stats.summary;

  // 활동 데이터 존재 여부 판단 (차트 데이터가 없거나 주요 앱이 "데이터 없음"으로 표시되는 경우)
  const hasNoData = !loading && (!stats.chartData || stats.chartData.length === 0 || summary.mainApp === "데이터 없음");

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

  // 활동 데이터 fetch 및 캐싱 로직
  useEffect(() => {
    const CACHE_KEY = "last_activity_fetch_time";
    const ONE_HOUR = 60 * 60 * 1000;
    
    const lastFetch = sessionStorage.getItem(CACHE_KEY);
    const now = Date.now();

    // 1시간 이내 기록이 있고 스토어에 데이터가 이미 있다면 API 호출 건너뜜
    if (lastFetch && (now - parseInt(lastFetch)) < ONE_HOUR && stats.chartData.length > 0) {
      return;
    }

    fetchAndAnalyze().then(() => {
      sessionStorage.setItem(CACHE_KEY, now.toString());
    });
  }, [fetchAndAnalyze, stats.chartData.length]);
  
  const renderSchedulePreview = () => {
    switch (viewMode) {
      case "일": return <ScheduleDay schedules={schedules} isPreview={true} currentDate={previewDate} setCurrentDate={setPreviewDate}/>;
      case "주": return <ScheduleWeek schedules={schedules} isPreview={true} />;
      case "월": return <ScheduleMonth schedules={schedules} isPreview={true} />;
      default: return <ScheduleDay schedules={schedules} isPreview={true} />;
    }
  };

  const handleMoveToSchedule = () => {
    const viewMap = { "일": "day", "주": "week", "월": "month" };
    const targetView = viewMap[viewMode] || "week";
    setActiveMenu("스케줄", targetView);
  };

  // 피드백 카드 클릭 시에만 ID를 저장하고 이동하도록 핸들러
  const handleMoveToFeedback = () => {
    if (latestId) {
      sessionStorage.setItem("target_session_id", latestId);
    }
    setActiveMenu("피드백");
  };

  // 마크다운 텍스트를 HTML로 변환하는 함수 (굵게, 기울임만 지원)
  const formatMarkdown = (text) => {
    if (!text) return "";
    return text
      .replace(/\*\*(.*?)\*\*/g, '<b>$1</b>') // 굵게
      .replace(/\*(.*?)\*/g, '<i>$1</i>');    // 기울임
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
              onClick={handleMoveToSchedule}
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
            <h4>최근 활동 요약</h4>
              {loading ? (
              // 1. 로딩 중일 때 표시
              <p>최근 작업 데이터를 분석 중입니다...</p>
            ) : hasNoData ? (
              // 2. 로딩 완료 후 데이터가 없을 때 표시
              <p className="empty-message">아직 활동 데이터가 존재하지 않습니다. 지금 바로 세션을 시작해 보세요!</p>
            ) : (
              // 3. 데이터가 있을 때 표시
              <p dangerouslySetInnerHTML={{ __html: summary.summarySentence }} />
            )}
          </div>
          
          <div className="card feedback-card" onClick={handleMoveToFeedback}>
            <h4>최근 작업 피드백</h4>
            <div className="feedback-highlight-container">
            {isFeedbackLoading ? (
                <p className="feedback-text loading-feedback">최근 세션의 피드백을 불러오는 중입니다...</p>
              ) : feedbackData ? (
                <>
                  <span className="feedback-main-title">
                    {feedbackData.summary_title || "종합 피드백"}
                  </span>
                  <p 
                    className="feedback-text"
                    dangerouslySetInnerHTML={{ 
                      __html: formatMarkdown(feedbackData.summary_description) 
                    }}
                  />
                </>
              ) : (
                <p className="feedback-text empty-feedback">
                  지금 작업 피드백을 확인해 보세요!
                </p>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* 2. 하단 섹션: 그래프 단독 배치 */}
      <div className="overview-bottom-section">
        <div className="card graph-full-card" onClick={() => setActiveMenu("활동 요약")}>
          <h4>최근 활동 그래프</h4>
          <div className="overview-graph-wrapper">
            <ActivityChart data={stats.chartData} />
          </div>
        </div>
      </div>
    </div>
  );
}