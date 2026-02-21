import React, { useState, useEffect } from "react";
import "./ScheduleWeek.css";

// 스케줄 주간 뷰 컴포넌트
export default function ScheduleWeek({ schedules, onScheduleClick }) {
  const [now, setNow] = useState(new Date()); // 현재 시간 표시선을 위한 상태

  // 1분마다 현재 시간 업데이트
  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(timer);
  }, []);

  const today = new Date(); // 오늘 날짜
  const currentWeekStart = new Date(today); // 이번 주 시작일 (일요일)
  currentWeekStart.setDate(today.getDate() - today.getDay()); // 일요일로 설정

  const weekDays = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(currentWeekStart);
    d.setDate(currentWeekStart.getDate() + i);
    return d;
  }); // 이번 주에 해당하는 7일 배열 생성

  const hours = Array.from({ length: 24 }, (_, i) => i); // 하루에 해당하는 1시부터 24시까지 시간 배열

  // 현재 시간 선의 상단 위치 계산 (1시간 = 40px 기준)
  const currentTimeTop = (now.getHours() * 60 + now.getMinutes()) / 60 * 40;
  const currentDayStr = now.toISOString().split('T')[0];

  return (
    <div className="week-calendar">
      {/* 좌측 시간 컬럼 */}
      <div className="time-column">
        {/* 상단 '시간' 헤더 */}
        <div className="time-header">시간</div>
        {/* 시간 레이블 */}
        {hours.map((h) => (
          <div key={h} className="time-label">
            {h !== 23 && <span>{h + 1}</span>}
          </div>
        ))}
      </div>

      {/* 요일 + 본문 컬럼 */}
      <div className="day-columns">
        {weekDays.map((day, idx) => {
          // [수정] 오전 9시 이전 날짜 밀림 방지를 위해 로컬 시간대 기준으로 YYYY-MM-DD 추출
          const year = day.getFullYear();
          const month = String(day.getMonth() + 1).padStart(2, '0');
          const date = String(day.getDate()).padStart(2, '0');
          const dayStr = `${year}-${month}-${date}`;
          
          // 여러 날짜에 걸친 일정 표시를 위해 범위 기반 필터링 적용
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
                {/* 눈금 표시 (희미한 실선) */}
                {hours.map((h) => (
                  <div key={h} className="time-line"></div>
                ))}

                {/* 현재 시간 표시선 (오늘 날짜인 경우에만 표시) */}
                {dayStr === currentDayStr && (
                  <div 
                    className="current-time-line" 
                    style={{ 
                      top: `${currentTimeTop}px`,
                      /* 현재 요일 인덱스(idx)만큼 왼쪽으로 선을 연장하여 시간 컬럼 끝까지 닿게 함 */
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