import React, { useState, useEffect, useRef } from "react";
import "./ScheduleAddModal.css";
import { useScheduleStore } from '../ScheduleStore';
import { useTaskStore } from '../../Task/TaskStore';
import Papa from "papaparse"; // CSV 파싱 라이브러리

export default function ScheduleAddModal({ onClose }) {
  const addSchedule = useScheduleStore((state) => state.addSchedule);
  const { tasks, fetchTasks } = useTaskStore();
  const fileInputRef = useRef(null); // 파일 인풋 참조

  useEffect(() => { // 컴포넌트 마운트 시 최신 작업 목록 로드
    fetchTasks();
  }, [fetchTasks]);

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
    
    setForm((prev) => {
      const nextForm = { ...prev, [name]: value };
      
      if (name === "start_date") {
        nextForm.end_date = value;
      }
      
      return nextForm;
    });
  };

  // 시간 포맷을 HH:mm 형식으로 보정하는 유틸리티 (9:00 -> 09:00)
  const formatTime = (timeStr) => {
    if (!timeStr) return "";
    const cleanTime = timeStr.trim();
    if (cleanTime.length === 4 && cleanTime.includes(":")) {
      return `0${cleanTime}`;
    }
    return cleanTime;
  };

  // CSV 파일 처리 로직
  const handleCsvUpload = (e) => {
    const file = e.target.files[0];
    if (!file) return;

    Papa.parse(file, {
      header: true, // 첫 줄을 키값으로 사용
      skipEmptyLines: true,
      encoding: "EUC-KR",
      complete: (results) => {
        const { data } = results;
        
        if (data.length === 0) {
          console.warn("데이터가 비어있음");
          alert("CSV 파일에 데이터가 없습니다.");
          return;
        }

        const validSchedules = [];
        let skipCount = 0;

        data.forEach((row, index) => {
          // [디버깅 보완] 헤더의 BOM 문자 제거 및 공백 정제
          const cleanRow = {};
          Object.keys(row).forEach(key => {
            const cleanKey = key.replace(/^\ufeff/, "").trim();
            cleanRow[cleanKey] = row[key];
          });

          // 정제된 cleanRow를 사용하여 매칭 진행
          const csvTaskName = (cleanRow.task_name || "").trim();

          const matchedTask = tasks.find(
            (t) => {
              const isMatch = t.label.trim() === csvTaskName;
              return isMatch;
            }
          );

          // 이름(name)과 매칭된 task_id가 모두 존재하는 경우에만 유효 데이터로 간주
          if (cleanRow.name && matchedTask) {
            const newSchedule = {
              name: cleanRow.name,
              task_id: matchedTask.id, // 찾은 작업의 실제 ID 대입
              description: cleanRow.description || "",
              start_date: cleanRow.start_date || "",
              start_time: formatTime(cleanRow.start_time)|| "09:00",
              end_date: cleanRow.end_date || cleanRow.start_date || "",
              end_time: formatTime(cleanRow.end_time) || "10:00",
            };
            validSchedules.push(newSchedule);
          } else {
            console.warn(`데이터 유효성 검사 실패 (name 존재 여부: ${!!cleanRow.name}, matchedTask 존재 여부: ${!!matchedTask})`);
            skipCount++;
          }
        });

        if (validSchedules.length === 0) {
          alert("등록 가능한 유효한 데이터가 없습니다. 작업 이름이 정확한지 확인해 주세요.");
          return;
        }

        // 변환된 데이터 등록
        validSchedules.forEach((schedule, idx) => {
          addSchedule(schedule);
        });

        const skipMessage = skipCount > 0 ? ` (미매칭/누락 데이터 ${skipCount}건 제외)` : "";
        alert(`${validSchedules.length}개의 일정이 등록되었습니다.${skipMessage}`);
        onClose();
      },
      error: (err) => {
        console.error("CSV 파싱 에러:", err);
        alert("파일을 읽는 중 오류가 발생했습니다.");
      }
    });
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
    <div className="add-overlay">
      <div className="modal-content large">
        <h2>새 일정 추가</h2>
        
        <form onSubmit={handleSubmit} className="modal-form">
          <div className="form-group">
            <label>일정 이름</label>
            <input
              type="text"
              name="name"
              placeholder="일정 이름을 입력하세요"
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
              {tasks.map((task) => (
                <option key={task.id} value={task.id}>
                  {task.label}
                </option>
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
            <div className="footer-left">
              <input 
                type="file" 
                accept=".csv" 
                onChange={handleCsvUpload} 
                style={{ display: 'none' }} 
                ref={fileInputRef}
              />
              <button 
                type="button" 
                className="csv-btn" 
                onClick={() => fileInputRef.current.click()}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{marginRight: '6px'}}>
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                  <polyline points="14 2 14 8 20 8"></polyline>
                  <line x1="16" y1="13" x2="8" y2="13"></line>
                  <line x1="16" y1="17" x2="8" y2="17"></line>
                  <polyline points="10 9 9 9 8 9"></polyline>
                </svg>
                CSV로 일괄 등록
              </button>
            </div>
            <div className="footer-right">
              <button type="button" className="cancel-btn" onClick={onClose}>취소</button>
              <button type="submit" className="save-btn">일정 등록</button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}