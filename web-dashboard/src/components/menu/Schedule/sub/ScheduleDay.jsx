import React, { useEffect, useState } from "react";
import "./ScheduleDay.css";

// [수정] 오전 9시 이전 날짜 밀림 방지를 위해 로컬 시간대 기준으로 YYYY-MM-DD 추출
const getFormattedDateString = (date) => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

// 스케줄 일간 뷰 컴포넌트
export default function ScheduleDay({ schedules = [], onScheduleClick }) {
  const [currentDate, setCurrentDate] = useState(new Date()); // 오늘 날짜로 설정
  
  // 표시 대상 날짜의 '연-월-일' 문자열을 상태 변화 없이 계산
  const currentDisplayDateStr = getFormattedDateString(currentDate);
  
  // 실제 '오늘' 날짜의 '연-월-일' 문자열
  const todayStr = getFormattedDateString(new Date());

  // 오늘 날짜인지 여부를 판단함
  const isCurrentlyToday = currentDisplayDateStr === todayStr;

  // 요일 배열 정의
  const weekdays = ["일", "월", "화", "수", "목", "금", "토"];

  // 현재 날짜의 요일 계산
  const dayOfWeek = weekdays[currentDate.getDay()];

  // 토요일과 일요일에 대한 클래스 지정 (토요일: saturday, 일요일: sunday)
  const dayClass =
    currentDate.getDay() === 0
      ? "sunday"
      : currentDate.getDay() === 6
      ? "saturday"
      : "";

  // 한 시간당 높이를 60px로 설정
  const HOUR_HEIGHT = 60;
  
  // 현재 날짜일 때만 위치를 계산하고, 아니면 null로 설정
  const [currentPosition, setCurrentPosition] = useState(null); 

  // useEffect 1) 현재 시간에 따른 시간 표시선 위치 업데이트
  // currentDisplayDateStr 또는 todayStr 변경 시 useEffect 재실행
  useEffect(() => {
    const updatePosition = () => {
      if (!isCurrentlyToday) {   // isCurrentlyToday가 false이면 업데이트 중단
        setCurrentPosition(null);
        return;
      }  

      const now = new Date(); // 현재 시각
      const minutes = now.getHours() * 60 + now.getMinutes(); // 현재 시각을 분 단위로 변환
      const position = (minutes / 60) * HOUR_HEIGHT - 1; // 현재 시각에 따른 위치 계산 (-1px 보정)
      setCurrentPosition(position); // 위치 상태 업데이트
    };

    if (isCurrentlyToday) {  // 오늘 날짜일 때만 타이머를 설정
      updatePosition();
      const timer = setInterval(updatePosition, 60000);
      return () => clearInterval(timer);
    } else {
      setCurrentPosition(null); // 오늘 날짜가 아닐 경우 위치 초기화 및 타이머 없음 보장
      return () => {};
    }
  }, [currentDisplayDateStr, todayStr, isCurrentlyToday]);

  // 날짜 조작 함수 개선 (timestamp 사용)
  const navigateDay = (direction) => {
    const newDate = new Date(currentDate);
    // 현재 날짜의 밀리초를 가져와 24시간(86400000ms)을 더하거나 뺌
    newDate.setDate(newDate.getDate() + direction);
    setCurrentDate(newDate);
  };

  // 네비게이션 버튼 핸들러
  const handlePrevDay = () => navigateDay(-1); // 이전 날로 이동
  const handleNextDay = () => navigateDay(1); // 다음 날로 이동
  const handleToday = () => setCurrentDate(new Date()); // 오늘 날짜로 이동

  // 일정 필터링
  const daySchedules = schedules.filter(
    (s) => s.start_date === currentDisplayDateStr || s.end_date === currentDisplayDateStr
  );

  return (
    <div className="day-view">
      {/* 상단 날짜 헤더 */}
      <div className="day-header">
        {/* 좌측 영역: ◀, ▶, '오늘' 버튼을 모두 포함하며 동일 간격 배치 */}
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
            {currentDate.getFullYear()}년 {currentDate.getMonth() + 1}월{" "}
            {currentDate.getDate()}일 ({dayOfWeek})
          </span>
        </div>

        {/* 우측 영역: 비워두고 flex: 1로 중앙 정렬 보조 */}
        <div className="day-header-right">
          {/* 비움 */}
        </div>
      </div>

      {/* 본문 */}
      <div className="day-body">
        {/* 시간 라벨 컬럼 */}
        <div className="day-time-column">
          {/* 0시부터 23시까지 라벨을 생성합니다. */}
          {Array.from({ length: 24 }, (_, i) => (
            <div 
              key={i} 
              className="day-time-label" 
            >
              {/* span 태그로 감싸서 CSS에서 상대 위치 조정 */}
              <span>{i.toString().padStart(2, "0")}:00</span>
            </div>
          ))}
        </div>

        {/* 타임라인 및 일정 영역 */}
        <div className="day-timeline">
          {/* 현재 시간선: 오늘일 경우에만 렌더링 */}
          {isCurrentlyToday && currentPosition !== null && (
            <div
              className="current-time-line"
              style={{ top: `${currentPosition}px` }}
            ></div>
          )}

          {/* 시간 구분선 (0시부터 23시까지) */}
          {Array.from({ length: 24 }, (_, i) => (
            <div key={i} className="time-line"></div>
          ))}

          {/* 일정 */}
          {daySchedules.map((s) => {
            const [sh, sm] = s.start_time.split(":").map((v) => parseInt(v, 10));
            const [eh, em] = s.end_time.split(":").map((v) => parseInt(v, 10));
            const totalStart = sh * 60 + sm;
            const totalEnd = eh * 60 + em;
            const durationMinutes = totalEnd - totalStart; // 일정 지속 시간(분) 계산
  
            const top = (totalStart / 60) * HOUR_HEIGHT;
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
                {durationMinutes >= 20 && (
                  <div className="task-time">
                    {s.start_time.slice(0, 5)} ~ {s.end_time.slice(0, 5)}
                  </div>
                )}
      
                {/* 40분 이상인 경우에만 상세 설명 노출 */}
                {durationMinutes >= 40 && (
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