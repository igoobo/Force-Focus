import React, { useState, useEffect } from "react";
import useMainStore from "../../../MainStore";
import authApi from "../../../api/authApi";
import "./Feedback.css";
import html2canvas from "html2canvas";
import { jsPDF } from "jspdf";

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

  // 세션 목록 및 선택 관련 상태
  const [sessionList, setSessionList] = useState([]);
  const [sessionLoading, setSessionLoading] = useState(true);
  const [selectedSessionId, setSelectedSessionId] = useState("");


  // PDF 출력용 전체 데이터 렌더링 함수
  const renderPDFContent = () => {
    if (!data) return null;

    // PDF 내부 마크다운 태그 처리를 위한 내부 함수
    const formatPdfText = (text) => {
      if (!text) return "";
      return text
        .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
        .replace(/\n/g, "<br/>");
    };

    return (
      <div id="pdf-report-root" style={{ width: "800px", position: "absolute", left: "-9999px" }}>
        {/* 캡처 시 레이아웃 깨짐 방지를 위해 명시적 스타일 지정 */}
        <div className={`pdf-content-wrapper pdf-report-container ${isDarkMode ? "dark" : ""}`} 
             style={{ 
               width: "800px", 
               height: "auto", 
               overflow: "visible", 
               padding: "40px", 
               backgroundColor: isDarkMode ? "#0f172a" : "#ffffff", 
               color: isDarkMode ? "#f1f5f9" : "#1e293b", 
               boxSizing: "border-box" 
             }}>
          
          {/* 리포트 첫 페이지 헤더 */}
          <div className="pdf-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", borderBottom: "2px solid #4f46e5", paddingBottom: "20px", marginBottom: "30px" }}>
            <h1 className="pdf-logo" style={{ color: "#4f46e5", margin: 0 }}>Force-Focus</h1>
            <div className="pdf-session-info" style={{ textAlign: "right", fontSize: "12px" }}>
              <p>분석 ID: {selectedSessionId}_{new Date().toISOString().replace(/[-:T]/g, '').slice(0, 8) + '_' + new Date().toTimeString().split(' ')[0].replace(/:/g, '')}</p>
              <p>분석 일시: {new Date().toLocaleString()}</p>
            </div>
          </div>

          {/* 본문 시작 영역: 헤더와 겹치지 않도록 id 부여 */}
          <div id="pdf-main-body">
            {/* 1. 종합 분석 섹션 */}
            <section className="pdf-section" style={{ marginBottom: "50px" }}>
              <h2 className="section-title" style={{ borderLeft: "5px solid #4f46e5", paddingLeft: "15px", marginBottom: "20px" }}>1. 종합 분석 리포트</h2>
              <div className="pdf-grid" style={{ display: "grid", gridTemplateColumns: "1fr", gap: "20px" }}>
                <div style={{ marginBottom: "20px", lineHeight: "1.6" }} dangerouslySetInnerHTML={{ __html: formatPdfText(data.summary_description) }} />
                {data.summary_cards?.map((card, i) => (
                  <div key={i} className="pdf-card" style={{ padding: "20px", borderRadius: "12px", border: "1px solid #cbd5e1", marginBottom: "10px" }}>
                    <h3 style={{ color: "#4f46e5", marginTop: 0 }}>{card.title}</h3>
                    <ul style={{ paddingLeft: "20px" }}>
                      {card.items?.map((item, idx) => (
                        <li key={idx} style={{ marginBottom: "8px" }} dangerouslySetInnerHTML={{ __html: formatPdfText(item) }} /> 
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </section>

            {/* 2. 상세 집중도 분석 섹션 */}
            <section className="pdf-section" style={{ marginBottom: "50px" }}>
              <h2 className="section-title" style={{ borderLeft: "5px solid #4f46e5", paddingLeft: "15px", marginBottom: "20px" }}>2. 상세 집중도 분석</h2>
              <div style={{ padding: "25px", borderRadius: "12px", background: isDarkMode ? "#1e293b" : "#f8fafc", border: "1px solid #cbd5e1" }}>
                <h4 style={{ color: "#4f46e5" }}>{data.focus_insight_title} ({data.focus_badge})</h4>
                <p style={{ fontSize: "18px", fontWeight: "bold" }}>평균 집중 점수: {data.focus_score}점</p>
                <div style={{ lineHeight: "1.8", marginTop: "20px" }} dangerouslySetInnerHTML={{ __html: formatPdfText(data.focus_insight_content) }} />
                <div style={{ marginTop: "20px", display: "grid", gridTemplateColumns: "1fr 1fr", gap: "15px" }}>
                   <div style={{ padding: "10px", border: "1px dashed #cbd5e1" }}>최대 연속 몰입: {data.focus_stats?.max_continuous}</div>
                   <div style={{ padding: "10px", border: "1px dashed #cbd5e1" }}>인지적 임계점: {data.focus_stats?.threshold}</div>
                </div>
              </div>
            </section>

            {/* 3. 피로도 및 가이드 섹션 */}
            <section className="pdf-section">
              <h2 className="section-title" style={{ borderLeft: "5px solid #4f46e5", paddingLeft: "15px", marginBottom: "20px" }}>3. 피로도 및 개선 가이드</h2>
              <div style={{ padding: "20px", borderRadius: "12px", background: isDarkMode ? "#1e293b" : "#fff5f5", border: "1px solid #feb2b2", marginBottom: "30px" }}>
                <h4 style={{ color: "#e53e3e", marginTop: 0 }}>관리 지표: {data.fatigue_badge}</h4>
                <p dangerouslySetInnerHTML={{ __html: formatPdfText(data.fatigue_description) }} />
                <p style={{ marginTop: "10px", fontWeight: "bold" }}>방해 앱: {data.distraction_app} ({data.distraction_ratio}%)</p>
              </div>
              
              <div className="pdf-strategy-grid" style={{ display: "grid", gridTemplateColumns: "1fr", gap: "20px" }}>
                {data.recovery_strategies?.map((strategy, idx) => (
                  <div key={idx} style={{ padding: "20px", border: "1px solid #cbd5e1", borderRadius: "12px" }}>
                    <h4 style={{ color: "#4f46e5", marginTop: 0 }}>{strategy.title}</h4>
                    <ul style={{ paddingLeft: "20px" }}>
                      {strategy.items?.map((item, i) => (
                        <li key={i} style={{ marginBottom: "5px" }} dangerouslySetInnerHTML={{ __html: formatPdfText(item) }} />
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </section>
          </div>
        </div>
      </div>
    );
  };

  // PDF 다운로드 핸들러
  const handleDownloadPDF = async () => {
    try {
      if (!data) {
        console.error("출력할 데이터가 없습니다.");
        return;
      }

      const pdf = new jsPDF("p", "mm", "a4", true);
      const pdfWidth = pdf.internal.pageSize.getWidth();
      const pdfHeight = pdf.internal.pageSize.getHeight();
      
      const sideMargin = 15;
      const marginTop = 8;
      const marginBottom = 25; 
      const contentWidth = pdfWidth - (sideMargin * 2);

      // 다크모드 무관하게 라이트 모드 테마 강제 적용
      const canvasOptions = {
        scale: 2,
        useCORS: true,
        onclone: (clonedDoc) => {
          const reportRoot = clonedDoc.querySelector("#pdf-report-root");
          if (reportRoot) {
            // 다크모드 클래스 제거 및 라이트 모드 속성 강제 주입
            reportRoot.classList.remove("dark-mode");
            reportRoot.setAttribute("data-theme", "light");
            // 최상위 컨테이너 강제 색상 지정
            const container = reportRoot.querySelector(".pdf-content-wrapper");
            if (container) {
              container.classList.remove("dark");
              container.style.backgroundColor = "#ffffff";
              container.style.color = "#1e293b";
            }
            // 상세 집중도 섹션 등 개별 요소 색상 보정
            const sections = reportRoot.querySelectorAll(".pdf-section div");
            sections.forEach(div => {
              if (div.style.background.includes("rgb") || div.style.background.includes("#")) {
                div.style.background = "#f8fafc";
              }
            });
          }
        }
      };

      // 1. 헤더 준비
      const headerElement = document.querySelector("#pdf-report-root .pdf-header");
      const headerCanvas = await html2canvas(headerElement, canvasOptions);
      const fullHeaderImg = headerCanvas.toDataURL("image/jpeg", 0.8);
      const fullHeaderHeight = (headerCanvas.height * contentWidth) / headerCanvas.width;

      // 2. 섹션들 캡처 및 루프
      const sections = document.querySelectorAll("#pdf-main-body .pdf-section");
      let currentY = 15 + fullHeaderHeight + marginTop;
      
      pdf.addImage(fullHeaderImg, "JPEG", sideMargin, 15, contentWidth, fullHeaderHeight, undefined, 'FAST');

      for (let i = 0; i < sections.length; i++) {
        const canvas = await html2canvas(sections[i], canvasOptions);
        const imgData = canvas.toDataURL("image/jpeg", 0.7);
        const sectionHeightInPDF = (canvas.height * contentWidth) / canvas.width;

        const sectionText = sections[i].innerText;
        const isSpecialSection = sectionText.includes("2. 상세 집중도 분석") || sectionText.includes("3. 피로도 및 개선 가이드");
        const availableSpace = pdfHeight - marginBottom - currentY;
        const maxContentHeightPerPage = pdfHeight - marginBottom - 24.5;

        if ((isSpecialSection && i !== 0) || 
            (sectionHeightInPDF <= maxContentHeightPerPage && sectionHeightInPDF > availableSpace)) {
          pdf.addPage();
          drawMiniHeader(pdf);
          currentY = 24.5; 
        } else if (availableSpace < 10) {
          pdf.addPage();
          drawMiniHeader(pdf);
          currentY = 24.5;
        }

        let remainingHeight = sectionHeightInPDF;
        let sourceYOffset = 0;

        while (remainingHeight > 0.1) {
          const currentAvailableSpace = pdfHeight - marginBottom - currentY;
          let drawHeight = Math.min(remainingHeight, currentAvailableSpace);

          if (remainingHeight > currentAvailableSpace) {
            drawHeight -= 5; 
          }

          pdf.addImage(
            imgData, "JPEG", 
            sideMargin, currentY - sourceYOffset, 
            contentWidth, sectionHeightInPDF, 
            undefined, 'MEDIUM'
          );

          pdf.setFillColor(255, 255, 255);
          pdf.rect(0, 0, pdfWidth, currentY - 0.5, 'F'); 
          pdf.rect(0, pdfHeight - marginBottom, pdfWidth, marginBottom, 'F');
          
          if (pdf.internal.getNumberOfPages() === 1) {
            pdf.addImage(fullHeaderImg, "JPEG", sideMargin, 15, contentWidth, fullHeaderHeight, undefined, 'FAST');
          } else {
            drawMiniHeader(pdf);
          }

          remainingHeight -= drawHeight;
          
          if (remainingHeight > 0.5) { 
            sourceYOffset += drawHeight;
            pdf.addPage();
            drawMiniHeader(pdf);
            currentY = 24.5; 
          } else {
            currentY += drawHeight + marginTop; 
          }
        }
      }

      const totalPages = pdf.internal.getNumberOfPages();
      for (let i = 1; i <= totalPages; i++) {
        pdf.setPage(i);
        pdf.setFont("helvetica", "normal");
        pdf.setFontSize(10);
        pdf.setTextColor(150);
        pdf.text(`${i} / ${totalPages}`, pdfWidth / 2, pdfHeight - 12, { align: "center" });
      }

      function drawMiniHeader(doc) {
        doc.setDrawColor(111, 66, 193); 
        doc.setLineWidth(0.7);
        doc.line(sideMargin, 20, pdfWidth - sideMargin, 20);
        doc.setFont("helvetica", "bold"); doc.setFontSize(14); doc.setTextColor(111, 66, 193); 
        doc.text("Force-Focus", sideMargin, 15);
        doc.setFont("helvetica", "normal"); doc.setFontSize(9); doc.setTextColor(150);
        doc.text("Performance Analysis Report", pdfWidth - sideMargin, 15, { align: "right" });
      }

      pdf.save(`ForceFocus_Report_${selectedSessionId || "general"}_${new Date().toISOString().replace(/[-:T]/g, '').slice(0, 8) + '_' + new Date().toTimeString().split(' ')[0].replace(/:/g, '')}.pdf`);
    } catch (error) {
      console.error("PDF 생성 실패:", error);
    }
  };

  // 세션 선택 모달에서 세션 ID를 받아오는 로직
  useEffect(() => {
    const targetId = sessionStorage.getItem("target_session_id");

    if (targetId) {
      setSelectedSessionId(targetId);
    
      sessionStorage.removeItem("target_session_id");
    } else {
      setSelectedSessionId("");
   }
  }, []);

  useEffect(() => {
    isMounted.current = true;
    if (Object.keys(feedbackCache).length === 0) {
      fetchSessionList(isMounted);
    } else {
      setSessionLoading(false);
    }

    return () => {
      isMounted.current = false;
    };
  }, []); 

  // 텍스트 포맷팅 함수
  const formatText = (text) => {
    if (!text) return "";
    return text
      .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
      .replace(/\n/g, "<br/>");
  };

  // 아이콘 지정 함수
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

    const timeoutId = setTimeout(() => {
      if (isMounted.current) {
        setSessionLoading(false);
        console.warn("세션 로드 타임아웃: 20초 경과");
      }
    }, 20000);

    try {
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

  const isMounted = React.useRef(true);

  useEffect(() => {
    isMounted.current = true;
    fetchSessionList(isMounted);

    return () => {
      isMounted.current = false; 
    };
  }, []);

  // 2. 선택된 세션 ID 변경 시 데이터 패칭
  useEffect(() => {
    if (!selectedSessionId) return;

    const fetchFeedbackData = async () => {
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
        const response = await authApi.get(`/api/v1/insight/analyze/${selectedSessionId}`);
        const freshData = response.data;
        
        setData(freshData);
        setFeedbackCache({ ...feedbackCache, [selectedSessionId]: freshData });
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

  const handleTabClick = (tabName) => {
    setFeedbackViewMode(tabName);
    if (tabName === "피로도" && data) {
      setProgressWidth(0);
      setTimeout(() => setProgressWidth(data.distraction_ratio || 0), 50);
    }
  };

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
              {data.summary_cards?.map((card, index) => (
                <div key={index} className={`detail-card ${index === 0 ? 'summary' : index === 1 ? 'evaluation' : 'improvement'}`}>
                  <h4>{card.title}</h4>
                  <ul>
                    {card.items?.map((item, idx) => (
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
                <span className="value">{data.focus_stats?.max_continuous}</span>
              </div>
              <div className="stat-item">
                <span className="label">인지적 임계점</span>
                <span className="value">{data.focus_stats?.threshold}</span>
              </div>
              <div className="stat-item">
                <span className="label">평균 집중도</span>
                <span className="value">{data.focus_stats?.average_score}</span>
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
            <div className="description" dangerouslySetInnerHTML={{ __html: formatText(data.fatigue_description) }} />
            
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
                    className={`session-item-card ${index === 0 ? 'latest' : ''}`}
                    onClick={() => setSelectedSessionId(session.id)}
                  >
                    <div className="session-info-group">
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

      {selectedSessionId && (
        <>
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
      
            <div className="feedback-header-actions">
              <button className="action-btn pdf-btn" onClick={handleDownloadPDF}>
                <span className="icon">📄</span>
                <span className="btn-text">PDF로 저장하기</span>
              </button>
              <button className="action-btn back-btn" onClick={() => setSelectedSessionId("")}>
                <span className="icon">↺</span>
                <span className="btn-text">다른 세션 선택</span>
              </button>
            </div>
          </div>

          <div className="feedback-content">
            {renderContent()}
          </div>
        </>
      )}
      {/* PDF 전용 렌더링 영역 (절대 좌표로 숨김) */}
      <div style={{ position: "absolute", top: "-9999px", left: "-9999px" }}>
        {renderPDFContent()}
      </div>
    </div>
  );
}