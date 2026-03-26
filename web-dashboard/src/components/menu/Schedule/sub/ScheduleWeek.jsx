import React, { useState, useEffect } from "react";
import "./ScheduleWeek.css";

// 스케줄 주간 뷰 컴포넌트
export default function ScheduleWeek({ schedules, onScheduleClick }) {
  const [now, setNow] = useState(new Date()); // 현재 시간 표시선을 위한 상태

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(timer);
  }, []);

  const today = new Date();
  const currentWeekStart = new Date(today); // 이번 주 시작일 (일요일)
  currentWeekStart.setDate(today.getDate() - today.getDay()); // 일요일로 설정

  const weekDays = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(currentWeekStart);
    d.setDate(currentWeekStart.getDate() + i);
    return d;
  }); // 이번 주에 해당하는 7일 배열

  const hours = Array.from({ length: 24 }, (_, i) => i); // 하루에 해당하는 1시부터 24시까지 시간 배열

  // 현재 시간 선의 상단 위치 계산 (1시간 = 40px 기준)
  const currentTimeTop = (now.getHours() * 60 + now.getMinutes()) / 60 * 40;
  
  // 현재 날짜를 YYYY-MM-DD 형식으로 변환 (로컬 시간대 기준)
  const currentYear = now.getFullYear();
  const currentMonth = String(now.getMonth() + 1).padStart(2, '0');
  const currentDate = String(now.getDate()).padStart(2, '0');
  const currentDayStr = `${currentYear}-${currentMonth}-${currentDate}`;

  return (
    <div className="week-calendar">
      {/* 좌측 시간 컬럼 */}
      <div className="time-column">
        <div className="time-header">시간</div>
        {hours.map((h) => (
          <div key={h} className="time-label">
            {h !== 23 && <span>{h + 1}</span>}
          </div>
        ))}
      </div>

      {/* 요일 + 본문 컬럼 */}
      <div className="day-columns">
        {weekDays.map((day, idx) => {
          const year = day.getFullYear();
          const month = String(day.getMonth() + 1).padStart(2, '0');
          const date = String(day.getDate()).padStart(2, '0');
          const dayStr = `${year}-${month}-${date}`;
          
          const daySchedules = schedules.filter((s) => 
            dayStr >= s.start_date && dayStr <= s.end_date
          );

          return (
            <div key={dayStr} className="day-column">
              {/* 요일 헤더 */}
              <div
                className={`day-header-cell ${
                  day.getDay() === 0
                    ? "sunday"
                    : day.getDay() === 6
                    ? "saturday"
                    : ""
                }`}
              >
                <div className="week-header-day">
                  {["일", "월", "화", "수", "목", "금", "토"][day.getDay()]}
                </div>
                <div className="week-header-date">
                  {day.getMonth() + 1}/{day.getDate()}
                </div>
              </div>

              {/* 본문 영역 */}
              <div className="day-body">
                {/* 눈금 표시 */}
                {hours.map((h) => (
                  <div key={h} className="time-line"></div>
                ))}

                {/* 현재 시간 표시선 (오늘 날짜인 경우에만 표시) */}
                {dayStr === currentDayStr && (
                  <div 
                    className="current-time-line" 
                    style={{ 
                      top: `${currentTimeTop}px`,
                      left: `calc(-60px - ${idx * 100}%)`,
                      width: `calc(100% + 60px + ${idx * 100}%)`
                    }}
                  >
                    <div className="current-time-pointer" />
                  </div>
                )}

                {daySchedules.map((s) => {
                  const startParts = s.start_time.split(":");
                  const endParts = s.end_time.split(":");
  
                  const startTotal = parseInt(startParts[0]) * 60 + parseInt(startParts[1]);
                  const endTotal = parseInt(endParts[0]) * 60 + parseInt(endParts[1]);
                  const durationMinutes = endTotal - startTotal; // 일정 지속 시간(분) 계산

                  const top = (startTotal / 60) * 40; // 1시간 = 40px
                  const height = (durationMinutes / 60) * 40;

                  return (
                    <div
                      key={s.id}
                      className="schedule-block"
                      style={{ top: `${top}px`, height: `${height}px`, cursor: "pointer" }}
                      onClick={() => onScheduleClick && onScheduleClick(s)}
                    >
                      {/* 15분 이상의 높이가 확보되어야 제목 노출 */}
                      {height >= 15 ? (
                        <>
                          <div className="task-title">{s.name}</div>
                          {/* 1시간(약 40px) 이상의 높이가 확보되어야 시간 노출 */}
                          {height >= 40 && (
                            <div className="task-time">
                              {s.start_time.slice(0, 5)} ~ {s.end_time.slice(0, 5)}
                            </div>
                          )}
                        </>
                      ) : null}
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}