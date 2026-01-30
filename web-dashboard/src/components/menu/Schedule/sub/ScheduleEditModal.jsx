import React, { useState, useEffect } from "react";
import "./ScheduleEditModal.css";
import { useScheduleStore } from "../ScheduleStore";
import { useTaskStore } from '../../Task/TaskStore';

export default function ScheduleEditModal({ schedule, onClose }) {
  const { updateSchedule } = useScheduleStore();
  const { tasks, fetchTasks } = useTaskStore();

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const [formData, setFormData] = useState({
    ...schedule,
    task_id: schedule.task_id || "" // 기존 연결된 작업이 있으면 로드
  });

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
  
    updateSchedule(formData); 
  
    alert("일정이 수정되었습니다.");
    onClose();
  };

  return (
    <div className="modal-overlay">
      <div className="modal-content large">
        <div className="modal-header">
          <h2>일정 수정</h2>
        </div>
        <form onSubmit={handleSubmit} className="modal-form">
          <div className="form-group">
            <label>일정 이름</label>
            <input
              type="text"
              name="name"
              value={formData.name}
              onChange={handleChange}
              required
            />
          </div>

          {/* 작업 선택 섹션 추가 */}
          <div className="form-group">
            <label>작업 종류</label>
            <select 
              name="task_id" 
              value={formData.task_id} 
              onChange={handleChange} 
              required
            >
              <option value="">-- 작업 종류를 선택하세요 --</option>
              {tasks.map(task => (
                <option key={task.id} value={task.id}>{task.label}</option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label>설명</label>
            <textarea
              name="description"
              value={formData.description}
              onChange={handleChange}
            />
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>시작 날짜</label>
              <input type="date" name="start_date" value={formData.start_date} onChange={handleChange} required />
            </div>
            <div className="form-group">
              <label>시작 시간</label>
              <input type="time" name="start_time" value={formData.start_time} onChange={handleChange} required />
            </div>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>종료 날짜</label>
              <input type="date" name="end_date" value={formData.end_date} onChange={handleChange} required />
            </div>
            <div className="form-group">
              <label>종료 시간</label>
              <input type="time" name="end_time" value={formData.end_time} onChange={handleChange} required />
            </div>
          </div>

          <div className="modal-footer">
            <button type="button" className="cancel-btn" onClick={onClose}>취소</button>
            <button type="submit" className="save-btn">변경사항 저장</button>
          </div>
        </form>
      </div>
    </div>
  );
}