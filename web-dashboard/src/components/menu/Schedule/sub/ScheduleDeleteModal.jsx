import React, { useState, useEffect } from "react";
import "./ScheduleDeleteModal.css";
import { useScheduleStore } from "../ScheduleStore";
import { useTaskStore } from "../../Task/TaskStore";

export default function ScheduleDeleteModal({ onClose }) {
  const { schedules, deleteSchedule } = useScheduleStore();
  const [selectedIds, setSelectedIds] = useState([]);
  const { tasks, fetchTasks } = useTaskStore();

  useEffect(() => {
    fetchTasks(); // 배지 표시를 위한 최신 작업 리스트 로드
  }, [fetchTasks]);

  // task_id를 통해 작업 라벨을 찾는 함수
  const getTaskLabel = (task_id) => {
    const task = tasks.find(t => String(t.id) === String(task_id));
    return task ? task.label : "연결된 작업 없음";
  };

  // [추가] 체크박스 선택/해제 핸들러
  const handleCheckboxChange = (id) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]
    );
  };

  // [수정] 일괄 삭제 로직으로 변경
  const handleDelete = async () => {
    if (selectedIds.length === 0) {
      alert("삭제할 일정을 각각 선택하세요.");
      return;
    }

    const confirmed = window.confirm(`선택한 ${selectedIds.length}개의 일정을 정말 삭제하시겠습니까?`);
    
    if (confirmed) {
      try {
        // 모든 선택된 ID에 대해 삭제 프로세스 실행
        await Promise.all(selectedIds.map(id => deleteSchedule(id)));
        alert("선택한 일정들이 모두 삭제되었습니다.");
        setSelectedIds([]); // 선택 초기화
      } catch (error) {
        console.error("삭제 중 오류 발생:", error);
        alert("일부 일정 삭제에 실패했습니다.");
      }
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
                // [수정] 선택 여부 확인 로직 변경
                className={`delete-schedule-card ${selectedIds.includes(s.id) ? "selected" : ""}`}
              >
                <div className="delete-card-left">
                  <input
                    // [수정] radio -> checkbox로 변경
                    type="checkbox"
                    name="selectedSchedule"
                    value={s.id}
                    // [수정] 체크 여부 확인 로직 변경
                    checked={selectedIds.includes(s.id)}
                    onChange={() => handleCheckboxChange(s.id)}
                    className="delete-radio"
                  />
                  <div className="delete-schedule-info">
                    <div className="delete-title-row">
                      <h3 className="delete-schedule-name">{s.name}</h3>
                      <span className="delete-task-badge">{getTaskLabel(s.task_id)}</span>
                    </div>
                    <p className="delete-schedule-time">
                      {s.start_date} {s.start_time.slice(0, 5)} ~ {s.end_time.slice(0, 5)}
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
          {/* [수정] 버튼 텍스트 및 비활성화 조건 변경 */}
          <button 
            className="delete-main-btn" 
            onClick={handleDelete} 
            disabled={selectedIds.length === 0}
          >
            {selectedIds.length > 0 ? `${selectedIds.length}개 삭제` : "일괄 삭제"}
          </button>
          <button className="delete-cancel-btn" onClick={onClose}>닫기</button>
        </div>
      </div>
    </div>
  );
}