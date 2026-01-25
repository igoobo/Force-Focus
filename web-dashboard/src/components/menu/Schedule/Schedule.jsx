import React, { useState, useEffect } from "react";
import "./Schedule.css";
import { useScheduleStore } from './ScheduleStore';
import useMainStore from "../../../MainStore";

import ScheduleDay from "./sub/ScheduleDay";
import ScheduleWeek from "./sub/ScheduleWeek";
import ScheduleMonth from "./sub/ScheduleMonth";
import ScheduleList from "./sub/ScheduleList";
import ScheduleAddModal from "./sub/ScheduleAddModal";
import ScheduleDeleteModal from "./sub/ScheduleDeleteModal";
import ScheduleEditModal from "./sub/ScheduleEditModal";

// 스케줄 메뉴 컴포넌트
export default function Schedule() {
  const { schedules, loading, fetchSchedules, clearSchedules } = useScheduleStore(); // 현재 저장된 일정 가져오기
  const [isAddOpen, setIsAddOpen] = useState(false); // 일정 추가 모달 상태 (초기값 : 닫힘)
  const [isDeleteOpen, setIsDeleteOpen] = useState(false); // 일정 삭제 모달 상태 (초기값 : 닫힘)
  const [isEditOpen, setIsEditOpen] = useState(false); // 일정 수정 모달 상태 (초기값 : 닫힘)
  const [selectedSchedule, setSelectedSchedule] = useState(null); // 수정할 일정 정보 상태
  const scheduleInitialView = useMainStore((state) => state.scheduleInitialView); // 스케줄 초기 뷰 모드
  const clearScheduleInitialView = useMainStore((state) => state.clearScheduleInitialView); // 초기 뷰 모드 클리어 함수

  const viewMode = useMainStore((state) => state.scheduleViewMode); // 현재 뷰 모드 상태
  const setViewMode = useMainStore((state) => state.setScheduleViewMode); // 뷰 모드 설정 함수

  const currentUser = localStorage.getItem('userEmail'); // 현재 사용자 식별자

  // 일정 추가 모달 열기/닫기 함수
  const openAddModal = () => setIsAddOpen(true);
  const closeAddModal = () => setIsAddOpen(false);

  // 일정 삭제 모달 열기/닫기 함수
  const openDeleteModal = () => setIsDeleteOpen(true);
  const closeDeleteModal = () => setIsDeleteOpen(false);

  // 수정 모달 제어 함수
  const openEditModal = (schedule) => {
    setSelectedSchedule(schedule);
    setIsEditOpen(true);
  };
  const closeEditModal = () => {
    setSelectedSchedule(null);
    setIsEditOpen(false);
  };

  useEffect(() => {
    fetchSchedules();
    return () => clearSchedules();
  }, [fetchSchedules, clearSchedules, currentUser]); // currentUser가 바뀌면 다시 실행됨

  useEffect(() => {
    if (scheduleInitialView) {    // Overview에서 넘어온 예약된 뷰 모드가 있다면 즉시 반영
      setViewMode(scheduleInitialView);
      clearScheduleInitialView(); // 적용 후 예약 정보 초기화
    }
  }, [scheduleInitialView, clearScheduleInitialView]);

  // 로딩 중일 때 표시할 화면
  if (loading) {
    return (
      <div className="schedule-loading-container">
        <div className="loader"></div>
        <p>일정 정보를 불러오는 중입니다...</p>
      </div>
    );
  }

  return (
    <div className="schedule-container">
      <div className="schedule-header">
        <div className="view-buttons">
          <button
            className={viewMode === "day" ? "active" : ""}
            onClick={() => setViewMode("day")}
          >
            일
          </button>
          <button
            className={viewMode === "week" ? "active" : ""}
            onClick={() => setViewMode("week")}
          >
            주
          </button>
          <button
            className={viewMode === "month" ? "active" : ""}
            onClick={() => setViewMode("month")}
          >
            월
          </button>
          <button
            className={viewMode === "list" ? "active" : ""}
            onClick={() => setViewMode("list")}
          >
            목록
          </button>
        </div>

        <div className="action-buttons">
          <button className="add-btn" onClick={openAddModal}>
            <span>+</span> 일정 추가
          </button>
          <button className="delete-btn" onClick={openDeleteModal}>
            <span>−</span> 일정 삭제
          </button>
        </div>
      </div>

      {/* 각 뷰 컴포넌트에 onScheduleClick 프롭으로 수정 함수 전달 */}
      {viewMode === "day" && <ScheduleDay key="day" schedules={schedules} onScheduleClick={openEditModal} />}
      {viewMode === "week" && <ScheduleWeek key="week" schedules={schedules} onScheduleClick={openEditModal} />}
      {viewMode === "month" && <ScheduleMonth key="month" schedules={schedules} onScheduleClick={openEditModal} />}
      {viewMode === "list" && <ScheduleList key="list" schedules={schedules} onScheduleClick={openEditModal} />}

      {isAddOpen && <ScheduleAddModal onClose={closeAddModal} />}
      {isDeleteOpen && <ScheduleDeleteModal onClose={closeDeleteModal} />}
      
      {/* 수정 모달 렌더링 */}
      {isEditOpen && selectedSchedule && (
        <ScheduleEditModal 
          schedule={selectedSchedule} 
          onClose={closeEditModal} 
        />
      )}
    </div>
  );
}
