import React, { useState } from "react";
import "./Feedback.css"

export default function Feedback() {
  // 메뉴가 축소됨에 따라 탭 구성 변경
  const [activeTab, setActiveTab] = useState("종합");

  const handleTabClick = (tabName) => {
    setActiveTab(tabName);
  };

  const renderContent = () => {
    switch (activeTab) {
    case "종합":
      return (
        <div className="feedback-section active">
          <div className="section-header">
            <h3>종합 분석 보고서: <span className="highlight">효율적 사용자</span></h3>
            <span className="badge">Success Profile</span>
          </div>
          <p className="description">
            이번 세션 분석 결과, 귀하는 명확한 목표 의식을 바탕으로 핵심 도구를 제어하는 능력이 매우 탁월한 것으로 나타났습니다. 
            전체 작업 시간 중 <strong>Google Chrome</strong>을 활용한 자료 조사 및 문서화 비중이 43%에 달하며, 
            이는 업무의 본질에 집중하고 있다는 강력한 지표입니다. 특히 35분간 유지된 연속 딥 워크(Deep Work) 세션은 
            일반 사용자의 평균치를 훨씬 상회하는 상위 15% 수준의 높은 생산성 근력을 증명합니다.
            <br/><br/>
            다만, 인지적 피로가 누적되는 세션 후반부에 <strong>Discord</strong>를 포함한 소셜 미디어 앱으로의 전환 시도가 9회 포착되었습니다. 
            이러한 미세한 흐름의 끊김은 뇌가 휴식을 갈구할 때 나타나는 전형적인 현상이지만, 무의식적인 앱 실행은 오히려 
            도파민 회로를 자극하여 실제 회복을 방해할 수 있습니다. 현재의 높은 효율성을 유지하기 위해서는 집중력이 저하되는 
            평균 12분의 공백기에 소셜 앱 대신 화면을 벗어난 물리적 휴식을 취할 것을 강력히 권장합니다.
          </p>
      
          <div className="feedback-grid">
            <div className="detail-card summary">
              <h4>📊 활동 요약</h4>
              <ul>
                <li><strong>핵심 도구 :</strong> Google Chrome (43%)</li>
                <li><strong>집중 성과 :</strong> 세션 평균 77.5% 몰입</li>
                <li><strong>방해 요소 :</strong> Discord 실행 시도 9회</li>
              </ul>
            </div>
            <div className="detail-card evaluation">
              <h4>✅ 긍정적 평가</h4>
              <ul>
                <li><strong>업무 본질 집중 :</strong> 전체 작업의 절반 가까이를 핵심 툴에 할애하고 있습니다.</li>
                <li><strong>높은 집중 근력 :</strong> 35분 연속 딥 워크는 상위 15%의 생산성 수치입니다.</li>
              </ul>
            </div>
          </div>
        </div>
      );
      case "집중도":
        return (
          <div className="feedback-section active">
            <div className="section-header">
                <h3>딥 워크(Deep Work) 및 집중 사이클 분석</h3>
                <span className="badge">Cognitive Analysis</span>
            </div>
            <div className="stats-box centered">
                <div className="stat-item">
                    <span className="label">최대 연속 몰입 </span>
                    <span className="value">35분</span>
                </div>
                <div className="stat-item">
                    <span className="label">인지적 임계점 </span>
                    <span className="value">32분</span>
                </div>
                <div className="stat-item">
                    <span className="label">평균 집중도 </span>
                    <span className="value">77.5%</span>
                </div>
            </div>
            <div className="feedback-content-body">
                <p>
                    귀하의 인지적 데이터 분석 결과, 작업 시작 후 약 <strong>7분 이내</strong>에 심층 집중 상태(Flow)에 진입하는 매우 높은 몰입 가속도를 보유하고 있습니다. 
                    전체 세션 중 <strong>35분</strong>간 유지된 최대 몰입 구간은 정보 처리 능력이 극대화되는 황금 시간대였으나, 
                    이후 32분 지점에서 인지적 부하(Cognitive Load)가 급격히 상승하며 효율이 소폭 하락하는 현상이 관찰되었습니다.
                </p>
                <div className="insight-box highlight-border">
                    <h4>심층 분석 리포트</h4>
                    <p>
                        데이터상으로 포착된 12분의 이탈 시간은 단순한 산만함이 아니라, 뇌의 전두엽이 과부하를 식히기 위해 보내는 
                        필수적인 '디폴트 모드 네트워크(DMN)' 활성화 신호로 해석됩니다. 이 시점에서 의도적으로 작업을 중단하지 않을 경우, 
                        무의식적인 소셜 미디어 탐색이나 불필요한 탭 전환이 발생할 확률이 68% 이상 높아집니다.
                    </p>
                </div>
            </div>
          </div>
        );
      case "피로도":
        return (
          <div className="feedback-section active">
            <div className="section-header">
                <h3>디지털 피로도 및 방해 요소 관리</h3>
                <span className="badge">Fatigue Management</span>
            </div>
            <p className="description">
                이번 세션에서 <strong>총 13회</strong>의 방해 프로그램 실행 시도가 포착되었습니다. 특히 집중력이 저하되는 구간에서 발생하는 
                습관적인 <strong>Discord 클릭(비중 62.3%)</strong>은 뇌가 즉각적인 보상을 원하는 '도파민 루프'에 빠졌음을 의미합니다. 
                이러한 미세 방해는 단순한 시간 낭비를 넘어, 다시 업무에 몰입하기 위한 인지적 에너지를 급격히 소모시켜 누적 피로도를 높이는 주범입니다.
            </p>
            
            <div className="distraction-bar-container" style={{margin: '20px 0'}}>
              <span className="label" style={{fontSize: '0.9rem', color: 'var(--text-muted)'}}>방해 요소 점유율: Discord (62.3%)</span>
              <div className="progress-bar">
                <div className="progress-fill" style={{width: '62.3%'}}></div>
              </div>
            </div>

            <div className="insight-box highlight-border">
              <h4>🔋 피로 회복을 위한 AI 가이드</h4>
              <p>
                현재의 피로 누적 패턴을 끊어내기 위해 다음과 같은 <strong>회복 전략</strong>을 제안합니다:
              </p>
              <div className="strategy-grid">
                <div className="strategy-item">
                  <div className="icon">👁️</div>
                  <h5>20-20-20 룰</h5>
                  <p>20분마다 20피트(6m) 밖을 20초간 응시하여 시각적 긴장을 해소하세요.</p>
                </div>
                <div className="strategy-item">
                  <div className="icon">💧</div>
                  <h5>물리적 격리</h5>
                  <p>방해 앱 클릭 욕구가 생길 때, 자리에서 일어나 물 한 잔을 마시며 뇌를 환기하세요.</p>
                </div>
              </div>
              <p style={{marginTop: '20px', fontSize: '0.95rem', fontStyle: 'italic', color: 'var(--text-muted)'}}>
                * 단순한 의지력의 문제가 아닙니다. 뇌에 '진짜 휴식'을 제공하여 인지적 탄력성을 회복하는 것이 지속 가능한 생산성의 핵심입니다.
              </p>
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
              className={activeTab === tab ? "active" : ""} 
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