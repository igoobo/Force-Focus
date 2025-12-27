import React from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

export const activityData = [
  { day: "월", events: 120, inputIntensity: 450, topApp: "VS Code" },
  { day: "화", events: 210, inputIntensity: 890, topApp: "Google Chrome" },
  { day: "수", events: 150, inputIntensity: 620, topApp: "VS Code" },
  { day: "목", events: 300, inputIntensity: 1200, topApp: "Google Chrome" },
  { day: "금", events: 240, inputIntensity: 950, topApp: "IntelliJ" },
  { day: "토", events: 90, inputIntensity: 200, topApp: "Youtube" },
  { day: "일", events: 60, inputIntensity: 150, topApp: "Notion" },
];

export const getActivitySummary = () => {
  const maxEvents = Math.max(...activityData.map(d => d.events));
  const busiestDayObj = activityData.find(d => d.events === maxEvents);
  const busiestDay = busiestDayObj ? busiestDayObj.day : "";
  const mainApp = "Google Chrome";
  const intensityChange = "15% 상승";

  return {
    busiestDay: `${busiestDay}`,
    mainApp,
    intensityChange,
    avgFocusTime: "5시간 20분", 
    intensityLevel: "보통",
    summarySentence: `이번 주에는 <strong>${busiestDay}요일</strong>에 업무 효율이 가장 높았습니다. 평소보다 <strong>${mainApp}</strong>을 통한 문서 작업 비중이 높았으며, 전체적인 집중 강도는 지난주 대비 약 <strong>${intensityChange}</strong>했습니다.`
  };
};

const ActivityChart = () => {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <AreaChart data={activityData} margin={{ left: -20, right: 10, top: 10, bottom: 0 }}>
        <defs>
          <linearGradient id="colorPrimary" x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3}/>
            <stop offset="95%" stopColor="#3b82f6" stopOpacity={0}/>
          </linearGradient>
        </defs>
        <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="var(--border-color)" />
        <XAxis 
          dataKey="day" 
          axisLine={false} 
          tickLine={false} 
          tick={{fill: 'var(--text-muted)', fontSize: 12}}
          interval={0} 
          padding={{ left: 30, right: 0 }}
        />
        <YAxis hide />
        <Tooltip contentStyle={{ backgroundColor: 'var(--card-bg)', borderColor: 'var(--border-color)', borderRadius: '8px' }} />
        <Area type="monotone" dataKey="events" stroke="#3b82f6" fillOpacity={1} fill="url(#colorPrimary)" strokeWidth={2} />
      </AreaChart>
    </ResponsiveContainer>
  );
};

export default ActivityChart;