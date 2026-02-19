import React from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const ActivityChart = ({ data = [] }) => {
  const displayData = data.length > 0 ? data : [
    { day: "일", events: 0, duration: 0 },
    { day: "월", events: 0, duration: 0 },
    { day: "화", events: 0, duration: 0 },
    { day: "수", events: 0, duration: 0 },
    { day: "목", events: 0, duration: 0 },
    { day: "금", events: 0, duration: 0 },
    { day: "토", events: 0, duration: 0 },
  ];

  return (
    <ResponsiveContainer width="100%" height="100%">
      <AreaChart data={displayData} margin={{ left: -20, right: 10, top: 10, bottom: 0 }}>
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
        <Tooltip 
          contentStyle={{ 
            backgroundColor: 'var(--card-bg)', 
            borderColor: 'var(--border-color)', 
            borderRadius: '8px',
            color: 'var(--text-main)'
          }} 
          itemStyle={{ color: '#3b82f6' }}
          formatter={(value, name) => [
            name === "events" ? `${value}회` : `${value}분`, 
            name === "events" ? "활동량" : "집중 시간"
          ]}
        />
        <Area 
          type="monotone" 
          dataKey="events" 
          stroke="#3b82f6" 
          fillOpacity={1} 
          fill="url(#colorPrimary)" 
          strokeWidth={2} 
          animationDuration={1000}
        />
      </AreaChart>
    </ResponsiveContainer>
  );
};

export default ActivityChart;