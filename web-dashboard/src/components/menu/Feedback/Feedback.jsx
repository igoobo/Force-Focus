import React, { useState, useEffect } from "react";
import useMainStore from "../../../MainStore";
import authApi from "../../../api/authApi";
import "./Feedback.css";

export default function Feedback() {
  const feedbackViewMode = useMainStore((state) => state.feedbackViewMode);
  const setFeedbackViewMode = useMainStore((state) => state.setFeedbackViewMode);
  const isDarkMode = useMainStore((state) => state.isDarkMode); // 전역 상태에서 다크모드 여부 확인
  
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [progressWidth, setProgressWidth] = useState(0); // 애니메이션용 상태

  const formatText = (text) => {
    if (!text) return "";
    return text
      .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
      .replace(/\n/g, "<br/>");
  };

  // [수정] 제목 키워드에 따른 동적 아이콘 지정 함수 추가
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

  useEffect(() => {
    const fetchFeedback = async () => {
      const cachedFeedback = sessionStorage.getItem("last_ai_feedback");
      
      if (cachedFeedback) {
        const parsed = JSON.parse(cachedFeedback);
        setData(parsed);
        setLoading(false);
        // 캐시 데이터 로드 시에도 약간의 지연 후 게이지 애니메이션 실행
        setTimeout(() => setProgressWidth(parsed.distraction_ratio || 0), 100);
        return;
      }

      setLoading(true);
      setError(null);
      try {
        const response = await authApi.get("/api/v1/insight/last-session");
        setData(response.data);
        sessionStorage.setItem("last_ai_feedback", JSON.stringify(response.data));
        // 데이터 수신 후 애니메이션 실행
        setTimeout(() => setProgressWidth(response.data.distraction_ratio || 0), 100);
      } catch (err) {
        console.error("AI Insight Fetch Error:", err);
        setError("데이터를 불러오는 중 오류가 발생했습니다.");
      } finally {
        setLoading(false);
      }
    };

    fetchFeedback();
  }, []);

  const handleTabClick = (tabName) => {
    setFeedbackViewMode(tabName);
    // 탭 전환 시 피로도 탭이면 게이지 애니메이션 재초기화
    if (tabName === "피로도" && data) {
      setProgressWidth(0);
      setTimeout(() => setProgressWidth(data.distraction_ratio || 0), 50);
    }
  };

  if (loading) {
    return (
      <div className={`feedback-container ${isDarkMode ? "dark-theme" : ""}`}>
        <div className="feedback-content" style={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
          <div className="loader"></div>
          <p style={{ marginLeft: '15px', color: 'var(--text-muted)' }}>AI가 귀하의 활동을 심층 분석 중입니다...</p>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className={`feedback-container ${isDarkMode ? "dark-theme" : ""}`}>
        <div className="feedback-content">
          <p style={{ color: 'var(--text-muted)' }}>{error || "표시할 분석 데이터가 없습니다."}</p>
        </div>
      </div>
    );
  }

  const renderContent = () => {
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
        // [수정] 반드시 2개의 카드가 출력되도록 데이터 보완 로직 추가
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
                    {/* [수정] getStrategyIcon 함수를 통한 동적 아이콘 할당 */}
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
      <div className="feedback-content">
        {renderContent()}
      </div>
    </div>
  );
}