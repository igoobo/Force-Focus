import { create } from "zustand";
import { sessionApi } from "../../../api/sessionApi";

export const useActivityStore = create((set, get) => ({
  stats: {
    chartData: [],
    summary: {
      busiestDay: "-",
      mainApp: "-",
      avgFocusTime: "0시간 0분",
      intensityLevel: "-",
      summarySentence: ""
    }
  },
  loading: false,

  fetchAndAnalyze: async () => {
    const token = localStorage.getItem('accessToken');
    if (!token) {
        console.warn("인증 토큰이 없습니다. 로그인이 필요합니다.");
        return;
    }
    
    set({ loading: true });
    try {
      const now = new Date();
      // API 호출 범위 설정을 위한 7일 전 시점 변수 선언
      const sevenDaysAgo = new Date();
      sevenDaysAgo.setDate(now.getDate() - 7);
      
      const weekdays = ["일", "월", "화", "수", "목", "금", "토"];
    
      // 7일치 기본 맵 생성 (날짜 어긋남 방지 및 로컬 시간대 기준 YYYY-MM-DD 추출)
      const analysisMap = Array.from({ length: 7 }, (_, i) => {
        const d = new Date();
        d.setDate(now.getDate() - (6 - i));
        return {
          dateStr: `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`,
          dayName: weekdays[d.getDay()],
          duration: 0,
          eventsCount: 0
        };
      });

      // 데이터 요청
      const [sessionRes, eventRes] = await Promise.all([
        sessionApi.getSessions(200),
        sessionApi.getEvents(sevenDaysAgo.toISOString(), now.toISOString(), 1000)
      ]);

      const sessions = sessionRes.data || [];
      const events = eventRes.data || [];

      // 1. Event 객체의 session_id(UUID)를 기준으로 개수 집계
      const eventCountMap = events.reduce((acc, event) => {
        const sId = event.session_id; 
        if (sId) {
          acc[sId] = (acc[sId] || 0) + 1;
        }
        return acc;
      }, {});

      // 2. 세션 데이터를 순회하며 client_session_id를 연결고리로 사용
      sessions.forEach(s => {
        const d = new Date(s.start_time);
        const sDateStr = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
        const day = analysisMap.find(item => item.dateStr === sDateStr);
  
        if (day) {
          // 초 단위(duration) 누적
          day.duration += (Number(s.duration) || 0);
          
          // 핵심: session.client_session_id 필드와 event.session_id를 매칭
          // 백엔드 데이터에 해당 필드가 포함되어 있는지 확인하며 매칭을 수행함
          const connectionId = s.client_session_id;
          if (connectionId && eventCountMap[connectionId]) {
            day.eventsCount += eventCountMap[connectionId];
          }
        }
      });

      // 주요 앱 사용 통계 계산
      const appUsage = {};
      events.forEach(e => {
        const appName = e.app_name || "알 수 없는 앱";
        appUsage[appName] = (appUsage[appName] || 0) + 1;
      });

      const topApp = Object.entries(appUsage).sort((a, b) => b[1] - a[1])[0]?.[0] || "데이터 없음";

      // 분석 결과 도출
      const busiestDayObj = [...analysisMap].sort((a, b) => b.eventsCount - a.eventsCount)[0];
      const totalDuration = analysisMap.reduce((acc, curr) => acc + curr.duration, 0);
      const avgMinutes = Math.floor((totalDuration / 7) / 60);

      const isActualDataEmpty = topApp === "데이터 없음" || totalDuration === 0;

      // 차트 데이터 및 상태 업데이트
      const chartData = analysisMap.map(d => ({
        day: d.dayName,
        events: d.eventsCount,
        duration: Math.round(d.duration / 60) // 초 단위를 분 단위로 변환하되, 아주 짧은 기록도 반영되도록 반올림 처리 
      }));

      set({
        stats: {
          chartData: isActualDataEmpty ? [] : chartData, // 데이터가 없으면 차트 배열 비움
          summary: {
            busiestDay: isActualDataEmpty ? "-" : (busiestDayObj?.dayName || "-"),
            mainApp: topApp,
            avgFocusTime: isActualDataEmpty ? "0시간 0분" : `${Math.floor(avgMinutes / 60)}시간 ${avgMinutes % 60}분`,
            intensityLevel: isActualDataEmpty ? "-" : (avgMinutes > 180 ? "매우 높음" : avgMinutes > 120 ? "높음" : avgMinutes > 60 ? "보통" : "낮음"),
            summarySentence: isActualDataEmpty 
              ? "" 
              : `최근 7일간 <strong>${topApp}</strong>을(를) 중심으로 총 <strong>${Math.floor(avgMinutes / 60)}시간 ${avgMinutes % 60}분</strong>의 일평균 집중 시간을 기록하셨습니다. 특히 <strong>${busiestDayObj?.dayName || '특정'}요일</strong>에 가장 높은 몰입도를 보였으며, 전반적인 집중 강도는 <strong>'${avgMinutes > 180 ? "매우 높음" : avgMinutes > 120 ? "높음" : avgMinutes > 60 ? "보통" : "낮음"}'</strong> 수준으로 분석되었습니다.`
          }
        },
        loading: false
      });
    } catch (err) {
      console.error("Activity Analysis Error:", err);
      set({ loading: false });
    }
  }
}));