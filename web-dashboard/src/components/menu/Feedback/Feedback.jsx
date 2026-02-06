import React, { useState, useEffect } from "react";
import useMainStore from "../../../MainStore";
import authApi from "../../../api/authApi"; // 인터셉터가 적용된 공통 API 인스턴스
import "./Feedback.css";

export default function Feedback() {
  const feedbackViewMode = useMainStore((state) => state.feedbackViewMode);
  const setFeedbackViewMode = useMainStore((state) => state.setFeedbackViewMode);
  
  // 상태 관리: AI 응답 데이터, 로딩 상태, 에러 상태
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchFeedback = async () => {
      setLoading(true);
      setError(null);
      try {
        // 백엔드의 최신 세션 분석 엔드포인트 호출
        // 특정 세션 ID가 없다면 /last-session 엔드포인트를 호출하도록 구성 가능
        const response = await authApi.get("/api/v1/insight/last-session");
        setData(response.data);
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
  };

  if (loading) {
    return (
      <div className="feedback-container">
        <div className="feedback-content" style={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
          <div className="loader"></div>
          <p style={{ marginLeft: '15px' }}>AI가 귀하의 활동을 심층 분석 중입니다...</p>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="feedback-container">
        <div className="feedback-content">
          <p>{error || "표시할 분석 데이터가 없습니다."}</p>
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
              <h3>종합 분석 보고서: <span className="highlight">{data.summary_title}</span></h3>
              <div className="badge-wrapper">
                <span className="badge">{data.summary_badge}</span>
              </div>
            </div>
            <p className="description" dangerouslySetInnerHTML={{ __html: data.summary_description.replace(/\n/g, '<br/>') }} />
        
            <div className="feedback-grid">
              {data.summary_cards.map((card, index) => (
                <div key={index} className={`detail-card ${index === 0 ? 'summary' : index === 1 ? 'evaluation' : 'improvement'}`}>
                  <h4>{card.title}</h4>
                  <ul>
                    {card.items.map((item, idx) => (
                      <li key={idx} dangerouslySetInnerHTML={{ __html: item }} />
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
              <p dangerouslySetInnerHTML={{ __html: data.focus_insight_content.replace(/\n/g, '<br/>') }} />
            </div>
          </div>
        );
      case "피로도":
        return (
          <div className="feedback-section active" key="fatigue">
            <div className="section-header">
              <h3>디지털 피로도 및 방해 요소 관리</h3>
              <div className="badge-wrapper">
                <span className="badge">{data.fatigue_badge}</span>
              </div>
            </div>
            <p className="description" dangerouslySetInnerHTML={{ __html: data.fatigue_description.replace(/\n/g, '<br/>') }} />
            
            <div className="distraction-bar-container" style={{margin: '20px 0'}}>
              <span className="label" style={{fontSize: '0.9rem', color: 'var(--text-muted)'}}>
                방해 요소 점유율: {data.distraction_app} ({data.distraction_ratio}%)
              </span>
              <div className="progress-bar">
                <div className="progress-fill" style={{width: `${data.distraction_ratio}%`}}></div>
              </div>
            </div>

            <div className="insight-box highlight-border">
              <h4>🔋 피로 회복을 위한 AI 가이드</h4>
              <p>현재의 피로 누적 패턴을 끊어내기 위해 다음과 같은 <strong>회복 전략</strong>을 제안합니다:</p>
              <div className="strategy-grid">
                {data.recovery_strategies.map((strategy, index) => (
                  <div key={index} className="strategy-item">
                    <div className="icon">{index === 0 ? '👁️' : '💧'}</div>
                    <h5>{strategy.title}</h5>
                    <ul>
                      {strategy.items.map((item, idx) => (
                        <p key={idx} style={{ fontSize: '0.9rem', color: '#64748b', margin: '5px 0' }}>{item}</p>
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
    <div className="feedback-container">
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