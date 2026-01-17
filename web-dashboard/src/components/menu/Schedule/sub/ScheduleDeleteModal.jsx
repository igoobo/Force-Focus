import React, { useState, useEffect } from "react";
import "./ScheduleDeleteModal.css";
import { useScheduleStore } from "../ScheduleStore";

export default function ScheduleDeleteModal({ onClose }) {
  const { schedules, deleteSchedule } = useScheduleStore();
  const [selectedId, setSelectedId] = useState(null);
  const [taskSessions, setTaskSessions] = useState([]);

  // 작업 목록 로드 (배지 표시용)
  useEffect(() => {
    const savedSessions = localStorage.getItem('task-db-sessions');
    if (savedSessions) {
      setTaskSessions(JSON.parse(savedSessions));
    }
  }, []);

  // task_id를 통해 작업 라벨을 찾는 함수
  const getTaskLabel = (task_id) => {
  const task = taskSessions.find(t => t.id === task_id);
  return task ? task.label : "연결된 작업 없음";
  };

  const handleDelete = () => {
    if (!selectedId) {
      alert("삭제할 일정을 선택하세요.");
      return;
    }
    const confirmed = window.confirm("정말 삭제하시겠습니까?");
    if (confirmed) {
      deleteSchedule(selectedId);
      alert("선택한 일정이 삭제되었습니다.");
      setSelectedId(null);
    }
  };

  return (
    <div className="delete-modal-overlay">
      <div className="delete-modal">
        <h2 className="delete-modal-title">일정 삭제</h2>

        <div className="delete-schedule-list">
          {schedules.length === 0 ? (
            <p className="delete-empty-text">등록된 일정이 없습니다.</p>
          ) : (
            schedules.map((s) => (
              <label
                key={s.id}
                className={`delete-schedule-card ${selectedId === s.id ? "selected" : ""}`}
              >
                <div className="delete-card-left">
                  <input
                    type="radio"
                    name="selectedSchedule"
                    value={s.id}
                    checked={selectedId === s.id}
                    onChange={() => setSelectedId(s.id)}
                    className="delete-radio"
                  />
                  <div className="delete-schedule-info">
                    <div className="delete-title-row">
                      <h3 className="delete-schedule-name">{s.name}</h3>
                      {/* 작업 종류 배지 추가 */}
                      <span className="delete-task-badge">{getTaskLabel(s.task_id)}</span>
                    </div>
                    <p className="delete-schedule-time">
                      {s.start_date} {s.start_time} ~ {s.end_time}
                    </p>
                    {s.description && (
                      <p className="delete-schedule-desc">{s.description}</p>
                    )}
                  </div>
                </div>
              </label>
            ))
          )}
        </div>

        <div className="delete-modal-footer">
          <button className="delete-main-btn" onClick={handleDelete} disabled={!selectedId}>삭제</button>
          <button className="delete-cancel-btn" onClick={onClose}>닫기</button>
        </div>
      </div>
    </div>
  );
}