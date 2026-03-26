import React, { useEffect, useState } from "react";
import "./ScheduleDay.css";

// 오전 9시 이전 날짜 밀림 방지를 위해 로컬 시간대 기준으로 YYYY-MM-DD 추출
const getFormattedDateString = (date) => {
  if (!(date instanceof Date) || isNaN(date)) return ""; // 방어 코드 추가
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

// 스케줄 일간 뷰 컴포넌트
export default function ScheduleDay({ schedules = [], onScheduleClick, currentDate, setCurrentDate }) {
  const date = (currentDate instanceof Date && !isNaN(currentDate)) ? currentDate : new Date();

  const currentDisplayDateStr = getFormattedDateString(date);
  const todayStr = getFormattedDateString(new Date());
  const isCurrentlyToday = currentDisplayDateStr === todayStr;

  const weekdays = ["일", "월", "화", "수", "목", "금", "토"];
  const dayOfWeek = weekdays[date.getDay()];

  // 토요일과 일요일에 대한 클래스 지정
  const dayClass =
    date.getDay() === 0
      ? "sunday"
      : date.getDay() === 6
      ? "saturday"
      : "";

  const HOUR_HEIGHT = 80;
  const OFFSET_Y = 20;
  
  const [currentPosition, setCurrentPosition] = useState(null); 

  useEffect(() => {
    const updatePosition = () => {
      if (!isCurrentlyToday) {
        setCurrentPosition(null);
        return;
      }  

      const now = new Date();
      const minutes = now.getHours() * 60 + now.getMinutes();
      const position = (minutes / 60) * HOUR_HEIGHT + OFFSET_Y;
      setCurrentPosition(position); // 위치 상태 업데이트
    };

    if (isCurrentlyToday) {  // 오늘 날짜일 때만 타이머를 설정
      updatePosition();
      const timer = setInterval(updatePosition, 60000);
      return () => clearInterval(timer);
    } else {
      setCurrentPosition(null);
      return () => {};
    }
  }, [currentDisplayDateStr, todayStr, isCurrentlyToday]);

  // 날짜 조작 함수
  const navigateDay = (direction) => {
    if (typeof setCurrentDate !== 'function') return;
    
    setCurrentDate((prevDate) => {
      const baseDate = (prevDate instanceof Date && !isNaN(prevDate)) ? prevDate : new Date();
      const newDate = new Date(baseDate);
      newDate.setDate(baseDate.getDate() + direction);
      return newDate;
    });
  };

  // 네비게이션 버튼 핸들러
  const handlePrevDay = () => navigateDay(-1);
  const handleNextDay = () => navigateDay(1);
  const handleToday = () => {
    if (typeof setCurrentDate === 'function') {
      setCurrentDate(new Date());
    }
  };

  // 일정 필터링
  const daySchedules = schedules.filter(
    (s) => s.start_date === currentDisplayDateStr || s.end_date === currentDisplayDateStr
  );

  return (
    <div className="day-view">
      {/* 상단 날짜 헤더 */}
      <div className="day-header">
        <div className="day-header-left">
          <button className="nav-btn" onClick={handlePrevDay} title="이전 날">
            〈
          </button>
          <button className="today-btn" onClick={handleToday}>
            오늘
          </button>
          <button className="nav-btn" onClick={handleNextDay} title="다음 날">
            〉
          </button>
        </div>
        {/* 중앙 영역: 제목만 배치 */}
        <div className="day-header-center">
          <span className={`day-title ${dayClass}`}>
            {date.getFullYear()}년 {date.getMonth() + 1}월{" "}
            {date.getDate()}일 ({dayOfWeek})
          </span>
        </div>

        {/* 우측 영역 */}
        <div className="day-header-right">
        </div>
      </div>

      {/* 본문 */}
      <div className="day-body">
        {/* 시간 라벨 컬럼 */}
        <div className="day-time-column">
          {Array.from({ length: 24 }, (_, i) => (
            <div 
              key={i} 
              className="day-time-label" 
            >
              <span>{i.toString().padStart(2, "0")}:00</span>
            </div>
          ))}
        </div>

        {/* 타임라인 및 일정 영역 */}
        <div className="day-timeline">
          {/* 현재 시간선 */}
          {isCurrentlyToday && currentPosition !== null && (
            <div
              className="current-time-line"
              style={{ top: `${currentPosition}px` }}
            ></div>
          )}

          {/* 시간 구분선 (0시부터 23시까지 30분 간격) */}
          {Array.from({ length: 48 }, (_, i) => (
            <div key={i} className="time-line"></div>
          ))}

          {/* 일정 */}
          {daySchedules.map((s) => {
            const [sh, sm] = s.start_time.split(":").map((v) => parseInt(v, 10));
            const [eh, em] = s.end_time.split(":").map((v) => parseInt(v, 10));
            const totalStart = sh * 60 + sm;
            const totalEnd = eh * 60 + em;
            const durationMinutes = totalEnd - totalStart;
  
            const top = (totalStart / 60) * HOUR_HEIGHT + OFFSET_Y;
            const height = (durationMinutes / 60) * HOUR_HEIGHT;

            return (
              <div
                key={s.id}
                className="day-task"
                style={{ top: `${top}px`, height: `${height}px`, cursor: "pointer" }}
                onClick={() => onScheduleClick && onScheduleClick(s)}
              >
                <div className="task-title">{s.name}</div>
      
                {/* 20분 이상인 경우에만 시간 정보 노출 */}
                {durationMinutes >= 25 && (
                  <div className="task-time">
                    {s.start_time.slice(0, 5)} ~ {s.end_time.slice(0, 5)}
                  </div>
                )}
      
                {/* 40분 이상인 경우에만 상세 설명 노출 */}
                {durationMinutes >= 50 && (
                  <div className="task-desc">{s.description}</div>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}