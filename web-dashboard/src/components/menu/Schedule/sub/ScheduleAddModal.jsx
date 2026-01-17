import React, { useState, useEffect } from "react";
import "./ScheduleAddModal.css";
import { useScheduleStore } from '../ScheduleStore';

export default function ScheduleAddModal({ onClose }) {
  const addSchedule = useScheduleStore((state) => state.addSchedule);
  const [taskSessions, setTaskSessions] = useState([]);

  useEffect(() => {
    const savedSessions = localStorage.getItem('task-db-sessions');
    if (savedSessions) {
      setTaskSessions(JSON.parse(savedSessions));
    } else {
      setTaskSessions([]);
    }
  }, []);

  const [form, setForm] = useState({
    name: "",
    task_id: "",
    description: "",
    start_date: "",
    start_time: "",
    end_date: "",
    end_time: "",
  });

  const handleChange = (e) => {
    const { name, value } = e.target;
    setForm({ ...form, [name]: value });
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    if (!form.task_id) {
      alert("연결할 작업을 선택해 주세요.");
      return;
    }
    addSchedule(form);
    alert("일정이 정상적으로 추가되었습니다.");
    onClose();
  };

  return (
    <div className="modal-overlay">
      <div className="modal-content large">
        <div className="modal-header">
          <h2>새 일정 추가</h2>
        </div>
        <form onSubmit={handleSubmit} className="modal-form">
          <div className="form-group">
            <label>일정 이름</label>
            <input
              type="text"
              name="name"
              placeholder="일정의 이름을 입력하세요"
              value={form.name}
              onChange={handleChange}
              required
            />
          </div>

          <div className="form-group">
            <label>작업 종류</label>
            <select 
              name="task_id" 
              value={form.task_id} 
              onChange={handleChange} 
              required
            >
              <option value="">-- 작업 종류를 선택하세요 --</option>
              {taskSessions.map(task => (
                <option key={task.id} value={task.id}>{task.label}</option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label>상세 설명</label>
            <textarea
              name="description"
              placeholder="상세 설명을 입력하세요"
              value={form.description}
              onChange={handleChange}
            />
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>시작 날짜</label>
              <input type="date" name="start_date" value={form.start_date} onChange={handleChange} required />
            </div>
            <div className="form-group">
              <label>시작 시간</label>
              <input type="time" name="start_time" value={form.start_time} onChange={handleChange} required />
            </div>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>종료 날짜</label>
              <input type="date" name="end_date" value={form.end_date} onChange={handleChange} required />
            </div>
            <div className="form-group">
              <label>종료 시간</label>
              <input type="time" name="end_time" value={form.end_time} onChange={handleChange} required />
            </div>
          </div>

          <div className="modal-footer">
            <button type="button" className="cancel-btn" onClick={onClose}>취소</button>
            <button type="submit" className="save-btn">일정 등록</button>
          </div>
        </form>
      </div>
    </div>
  );
}