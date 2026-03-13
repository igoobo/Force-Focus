import React, { useState, useEffect } from "react";
import useMainStore from "../../../MainStore";
import authApi from "../../../api/authApi";
import "./Feedback.css";

export default function Feedback() {
  const feedbackViewMode = useMainStore((state) => state.feedbackViewMode);
  const setFeedbackViewMode = useMainStore((state) => state.setFeedbackViewMode);
  const isDarkMode = useMainStore((state) => state.isDarkMode); 
  
  // 전역 스토어 캐시 상태 및 업데이트 액션 구독
  const feedbackCache = useMainStore((state) => state.feedbackCache);
  const setFeedbackCache = useMainStore((state) => state.setFeedbackCache);

  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(false); 
  const [error, setError] = useState(null);
  const [progressWidth, setProgressWidth] = useState(0); 

  // --- 세션 목록 및 선택 관련 상태 ---
  const [sessionList, setSessionList] = useState([]);
  const [sessionLoading, setSessionLoading] = useState(true);
  const [selectedSessionId, setSelectedSessionId] = useState("");

  // old_feedback.jsx의 텍스트 포맷팅 함수
  const formatText = (text) => {
    if (!text) return "";
    return text
      .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
      .replace(/\n/g, "<br/>");
  };

  // old_feedback.jsx의 아이콘 지정 함수
  const getStrategyIcon = (title) => {
    if (!title) return "💡";
    if (title.includes("시각") || title.includes("눈") || title.includes("화면")) return "👁️";
    if (title.includes("신체") || title.includes("스트레칭") || title.includes("자세") || title.includes("근육")) return "🧘";
    if (title.includes("수분") || title.includes("물") || title.includes("차")) return "💧";
    if (title.includes("환경") || title.includes("환기") || title.includes("정리")) return "🌿";
    if (title.includes("명상") || title.includes("호흡") || title.includes("마음")) return "✨";
    if (title.includes("휴식") || title.includes("전략")) return "💡";
    return "💡";
  };

  // 1. 컴포넌트 마운트 시 세션 목록 로드
  const fetchSessionList = async (isMounted) => {
    setSessionLoading(true);

    // 20초 타임아웃 설정
    const timeoutId = setTimeout(() => {
      if (isMounted.current) {
        setSessionLoading(false);
        console.warn("세션 로드 타임아웃: 20초 경과");
      }
    }, 20000);

    try {
      // 캐시 방지를 위한 타임스탬프 추가
      const response = await authApi.get(`/api/v1/sessions/?limit=20&t=${new Date().getTime()}`);
      const sessions = Array.isArray(response.data) ? response.data : (response.data.sessions || []);
    
      if (isMounted.current) {
        clearTimeout(timeoutId);
        setSessionList(sessions);
      }
    } catch (err) {
      console.error("세션 목록 로드 실패:", err);
      if (isMounted.current) clearTimeout(timeoutId);
    } finally {
      if (isMounted.current) setSessionLoading(false);
    }
  };

  // 컴포넌트 마운트 시 실행되는 Hook
  const isMounted = React.useRef(true);

  useEffect(() => {
    isMounted.current = true;

    // 1 & 2번 조건: 전체 새로고침 시 컴포넌트가 재마운트되므로 항상 이 로직이 실행됨
    // 이전에 타임아웃이 났었더라도 새로고침하면 다시 호출함
    fetchSessionList(isMounted);

    return () => {
      isMounted.current = false; // 언마운트 시 처리
    };
  }, []);

  // 2. 선택된 세션 ID 변경 시 데이터 페칭 (캐싱 포함)
  useEffect(() => {
    if (!selectedSessionId) return;

    const fetchFeedbackData = async () => {
      // 캐시 확인
      if (feedbackCache[selectedSessionId]) {
        const cachedData = feedbackCache[selectedSessionId];
        setData(cachedData);
        setLoading(false);
        setTimeout(() => setProgressWidth(cachedData.distraction_ratio || 0), 100);
        return;
      }

      setLoading(true);
      setError(null);
      try {
        // 백엔드 엔드포인트: /insight/analyze/{id}
        const response = await authApi.get(`/api/v1/insight/analyze/${selectedSessionId}`);
        const freshData = response.data;
        
        setData(freshData);
        // 전역 스토어 캐시 업데이트
        setFeedbackCache({ ...feedbackCache, [selectedSessionId]: freshData });
        
        // 애니메이션 실행
        setTimeout(() => setProgressWidth(freshData.distraction_ratio || 0), 100);
      } catch (err) {
        console.error("AI Insight Fetch Error:", err);
        setError("데이터를 불러오는 중 오류가 발생했습니다.");
      } finally {
        setLoading(false);
      }
    };

    fetchFeedbackData();
  }, [selectedSessionId]);

  // old_feedback.jsx의 탭 클릭 로직
  const handleTabClick = (tabName) => {
    setFeedbackViewMode(tabName);
    if (tabName === "피로도" && data) {
      setProgressWidth(0);
      setTimeout(() => setProgressWidth(data.distraction_ratio || 0), 50);
    }
  };

  // old_feedback.jsx의 렌더링 로직 (데이터 구조 변경 없음)
  const renderContent = () => {
    if (loading) {
      return (
        <div className="feedback-content">
          <div className="feedback-loading-container">
            <div className="loader"></div>
            <p>세션 활동을 분석하고 있습니다...</p>
          </div>
        </div>
      );
    }

    if (error || !data) {
      return (
        <div className="feedback-content">
          <p style={{ color: 'var(--text-muted)', textAlign: 'center', padding: '40px' }}>
            {error || "표시할 분석 데이터가 없습니다."}
          </p>
        </div>
      );
    }

    switch (feedbackViewMode) {
      case "종합":
        return (
          <div className="feedback-section active" key="total">
            <div className="section-header">
              <h3>{data.summary_title}</h3>
              <div className="badge-wrapper">
                <span className="badge">{data.summary_badge}</span>
              </div>
            </div>
            <p className="description" dangerouslySetInnerHTML={{ __html: formatText(data.summary_description) }} />
            
            <div className="feedback-grid">
              {data.summary_cards.map((card, index) => (
                <div key={index} className={`detail-card ${index === 0 ? 'summary' : index === 1 ? 'evaluation' : 'improvement'}`}>
                  <h4>{card.title}</h4>
                  <ul>
                    {card.items.map((item, idx) => (
                      <li key={idx} dangerouslySetInnerHTML={{ __html: formatText(item) }} />
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          </div>
        );
      case "집중도":
        return (
          <div className="feedback-section active" key="focus">
            <div className="section-header">
              <h3>{data.focus_insight_title}</h3>
              <div className="badge-wrapper">
                <span className="badge">{data.focus_badge}</span>
              </div>
            </div>
            <div className="stats-box centered">
              <div className="stat-item">
                <span className="label">최대 연속 몰입</span>
                <span className="value">{data.focus_stats.max_continuous}</span>
              </div>
              <div className="stat-item">
                <span className="label">인지적 임계점</span>
                <span className="value">{data.focus_stats.threshold}</span>
              </div>
              <div className="stat-item">
                <span className="label">평균 집중도</span>
                <span className="value">{data.focus_stats.average_score}</span>
              </div>
            </div>
            <div className="feedback-content-body">
              <p dangerouslySetInnerHTML={{ __html: formatText(data.focus_insight_content) }} />
            </div>
          </div>
        );
      case "피로도":
        const displayStrategies = [...(data.recovery_strategies || [])];
        if (displayStrategies.length < 1) {
          displayStrategies.push({ title: "시각적 휴식", items: ["20-20-20 규칙을 실천하세요.", "먼 곳을 바라보며 눈의 근육을 이완시키세요."] });
        }
        if (displayStrategies.length < 2) {
          displayStrategies.push({ title: "신체 스트레칭", items: ["목과 어깨를 가볍게 돌려주세요.", "자리에서 일어나 가벼운 기지개를 켜세요."] });
        }
        const finalStrategies = displayStrategies.slice(0, 2);

        return (
          <div className="feedback-section active" key="fatigue">
            <div className="section-header">
              <h3>디지털 피로도 및 방해 요소 관리</h3>
              <div className="badge-wrapper">
                <span className="badge">{data.fatigue_badge}</span>
              </div>
            </div>
            <p className="description" dangerouslySetInnerHTML={{ __html: formatText(data.fatigue_description) }} />
            
            <div className="distraction-bar-container">
              <span className="label" style={{color: 'var(--text-muted)', textAlign: 'center'}}>
                방해 요소 점유율: <strong>{data.distraction_app}</strong> ({data.distraction_ratio}%)
              </span>
              <div className="progress-bar">
                <div className="progress-fill" style={{width: `${progressWidth}%`}}></div>
              </div>
            </div>

            <div className="insight-box highlight-border">
              <h4 style={{textAlign: 'center'}}>🔋 피로 회복을 위한 AI 가이드</h4>
              <p style={{textAlign: 'center'}}>현재의 피로 누적 패턴을 끊어내기 위해 다음과 같은 <strong>회복 전략</strong>을 제안합니다.</p>
              <div className="strategy-grid">
                {finalStrategies.map((strategy, index) => (
                  <div key={index} className="strategy-item">
                    <div className="icon" style={{textAlign: 'center', width: '100%'}}>
                      {getStrategyIcon(strategy.title)}
                    </div>
                    <h5 style={{color: 'var(--text-main)', margin: '10px 0', textAlign: 'left', width: '100%'}}>
                      {strategy.title}
                    </h5>
                    <ul style={{padding: '0 0 0 18px', listStyle: 'disc', textAlign: 'left', width: '100%'}}>
                      {strategy.items.map((item, idx) => (
                        <li 
                          key={idx} 
                          style={{ fontSize: '0.9rem', color: 'var(--text-muted)', margin: '5px 0', textAlign: 'left' }}
                          dangerouslySetInnerHTML={{ __html: formatText(item) }} 
                        />
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className={`feedback-container ${isDarkMode ? "dark-theme" : ""}`}>
      {/* 세션 선택 모달 UI */}
      {!selectedSessionId && (
        <div className="modal-overlay">
          <div className="session-selection-modal fade-in">
            <div className="modal-header">
              <h2>피드백 대상 세션 선택</h2>
              <p>AI 기반 피드백을 확인하고 싶은 작업 세션을 선택해 주세요.</p>
            </div>
            <div className="session-list-wrapper">
              {sessionLoading ? (
                <div className="session-status-container">
                  <div className="loader"></div>
                  <p className="empty-msg">세션 데이터를 불러오는 중입니다...</p>
                </div>
              ) : sessionList.length > 0 ? (
                sessionList.map((session, index) => (
                  <div 
                    key={session.id} 
                    /* 첫 번째 아이템(가장 최근)에 'latest' 클래스 부여 */
                    className={`session-item-card ${index === 0 ? 'latest' : ''}`}
                    onClick={() => setSelectedSessionId(session.id)}
                  >
                    <div className="session-info-group">
                      {/* 가장 최근 세션일 경우 [최근] 문구 표시 */}
                      {index === 0 && <span className="latest-badge">최근</span>}
                      <span className="session-date">
                        {new Date(session.start_time).toLocaleString('ko-KR', { 
                          month: 'long', day: 'numeric', hour: '2-digit', minute: '2-digit' 
                        })} 세션
                      </span>
                    </div>
                    <span className="arrow-icon">→</span>
                  </div>
                ))
              ) : (
                // 3. 세션 기록이 아예 없는 경우
                <div className="session-status-container">
                  <p className="empty-msg">불러올 사용자 세션 기록이 없습니다.</p>
                  <button 
                    className="general-feedback-btn"
                    onClick={() => setSelectedSessionId("general")}
                  >
                    기본 가이드(범용 피드백) 열람하기
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* 선택 후 보여질 피드백 레이아웃 */}
      {selectedSessionId && (
        <>
          {/* 버튼을 메뉴 바 안으로 이동하여 높이를 통일함 */}
          <div className="feedback-menu-container">
            <div className="feedback-menu">
              <ul>
                {["종합", "집중도", "피로도"].map((tab) => (
                  <li 
                    key={tab}
                    className={feedbackViewMode === tab ? "active" : ""} 
                    onClick={() => handleTabClick(tab)}
                  >
                    {tab}
                  </li>
                ))}
              </ul>
            </div>
      
            {/* 우측 정렬된 다른 세션 선택 버튼 */}
            <button className="back-to-list-btn" onClick={() => setSelectedSessionId("")}>
              <span className="icon">↺</span> 다른 세션 선택
            </button>
          </div>

          <div className="feedback-content">
            {renderContent()}
          </div>
        </>
      )}
    </div>
  );
}