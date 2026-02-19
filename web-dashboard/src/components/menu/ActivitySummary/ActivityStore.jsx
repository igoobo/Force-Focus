// src/store/ActivityStore.jsx
import { create } from "zustand";
import { sessionApi } from "../../../api/sessionApi";

export const useActivityStore = create((set, get) => ({
  stats: {
    chartData: [],
    summary: {
      busiestDay: "-",
      mainApp: "-",
      avgFocusTime: "0시간",
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
      const sevenDaysAgo = new Date();
      sevenDaysAgo.setDate(now.getDate() - 6);
      sevenDaysAgo.setHours(0, 0, 0, 0);

      const [sessionRes, eventRes] = await Promise.all([
        sessionApi.getSessions(200),
        sessionApi.getEvents(sevenDaysAgo.toISOString(), now.toISOString(), 1000)
      ]);

      const sessions = sessionRes.data;
      const events = eventRes.data;

      const days = ["일", "월", "화", "수", "목", "금", "토"];
      const analysisMap = Array.from({ length: 7 }, (_, i) => {
        const d = new Date(sevenDaysAgo);
        d.setDate(sevenDaysAgo.getDate() + i);
        return {
          dateStr: d.toISOString().split('T')[0],
          dayName: days[d.getDay()],
          eventsCount: 0,
          duration: 0,
        };
      });

      sessions.forEach(s => {
        const sDate = s.start_time.split('T')[0];
        const day = analysisMap.find(d => d.dateStr === sDate);
        if (day) {
          day.duration += s.duration || 0;
          day.eventsCount += s.event_count || 0;
        }
      });

      const appUsage = {};
      events.forEach(e => {
        const appName = e.app_name || "알 수 없는 앱";
        appUsage[appName] = (appUsage[appName] || 0) + 1;
      });

      const topApp = Object.entries(appUsage).sort((a, b) => b[1] - a[1])[0]?.[0] || "데이터 없음";

      const busiestDayObj = [...analysisMap].sort((a, b) => b.eventsCount - a.eventsCount)[0];
      const totalDuration = analysisMap.reduce((acc, curr) => acc + curr.duration, 0);
      const avgMinutes = Math.floor((totalDuration / 7) / 60);

      const chartData = analysisMap.map(d => ({
        day: d.dayName,
        events: d.eventsCount,
        duration: Math.floor(d.duration / 60)
      }));

      set({
        stats: {
          chartData,
          summary: {
            busiestDay: busiestDayObj.dayName,
            mainApp: topApp,
            avgFocusTime: `${Math.floor(avgMinutes / 60)}시간 ${avgMinutes % 60}분`,
            intensityLevel: avgMinutes > 180 ? "매우 높음" : avgMinutes > 120 ? "높음" : avgMinutes > 60 ? "보통" : "낮음",
            summarySentence: `지난 7일간 <strong>${topApp}</strong>을 가장 많이 활용하셨으며, <strong>${busiestDayObj.dayName}요일</strong>에 가장 높은 몰입도를 보이셨습니다.`
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