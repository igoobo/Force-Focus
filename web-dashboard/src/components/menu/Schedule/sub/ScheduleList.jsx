import React, { useState, useEffect } from "react";
import "./ScheduleList.css";
import { useTaskStore } from "../../Task/TaskStore";

const ScheduleList = ({ schedules = [], onScheduleClick }) => {
  const { tasks, fetchTasks } = useTaskStore();

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const sortedSchedules = [...schedules].sort(
    (a, b) => new Date(b.start_date + " " + b.start_time) - new Date(a.start_date + " " + a.start_time)
  );

  // task_id를 통해 작업 라벨을 찾는 함수
  const getTaskLabel = (task_id) => {
    const task = tasks.find(t => String(t.id) === String(task_id));
    return task ? task.label : "연결된 작업 없음";
  };

  // CSV 내보내기 함수
  const handleExportCsv = () => {
    if (schedules.length === 0) {
      alert("내보낼 일정 데이터가 없습니다.");
      return;
    }

    // 1. 헤더 정의 (불러오기 형식과 동일)
    const headers = ["name", "task_name", "description", "start_date", "start_time", "end_date", "end_time"];
    
    // 2. 데이터 변환
    const csvRows = sortedSchedules.map(item => {
      const row = [
        item.name,
        getTaskLabel(item.task_id), // ID 대신 텍스트 작업명
        item.description || "",
        item.start_date,
        item.start_time.slice(0, 5),
        item.end_date,
        item.end_time.slice(0, 5)
      ];
      // 쉼표 포함 대비하여 각 항목을 큰따옴표로 감쌈
      return row.map(val => `"${String(val).replace(/"/g, '""')}"`).join(",");
    });

    // 3. BOM 추가 (한글 깨짐 방지) 및 결합
    const csvContent = "\ufeff" + headers.join(",") + "\n" + csvRows.join("\n");
    
    // 4. 다운로드 링크 생성 및 실행
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.setAttribute("download", `schedule_export_${new Date().toISOString().replace(/[-:T]/g, '').slice(0, 8) + '_' + new Date().toTimeString().split(' ')[0].replace(/:/g, '')}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div className="schedule-list-container">
        {/* 제목과 버튼을 하나의 섹션으로 그룹화 */}
        <div className="list-header-section">
          <div className="header-text">
            <h2 className="list-title">일정 목록</h2>
          </div>
          <button className="export-csv-btn" onClick={handleExportCsv}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            CSV로 내보내기
          </button>
        </div>
        <p className="list-subtitle">지금까지 추가한 일정을 조회할 수 있습니다.</p>

      {sortedSchedules.length === 0 ? (
        <div className="empty-list">등록된 일정이 없습니다.</div>
      ) : (
        <div className="schedule-card-list">
          {sortedSchedules.map((item) => (
            <div 
              key={item.id} 
              className="schedule-card"
              onClick={() => onScheduleClick && onScheduleClick(item)}
            >
              <div className="card-header">
                <div className="title-row">
                  <h3 className="card-title">{item.name}</h3>
                  <span className="task-badge">{getTaskLabel(item.task_id)}</span>
                </div>
                <span className="card-date">
                  {item.start_date} {item.start_time.slice(0, 5)} ~ {item.end_date} {item.end_time.slice(0, 5)}
                </span>
              </div>

              <p className="card-description">{item.description}</p>

              <div className="card-footer">
                <span className="created-at">
                  생성일: {item.created_at ? 
                    new Date(item.created_at.endsWith('Z') ? item.created_at : item.created_at.replace(' ', 'T') + 'Z')
                      .toLocaleDateString("ko-KR", { 
                        year: 'numeric', 
                        month: '2-digit', 
                        day: '2-digit',
                        timeZone: "Asia/Seoul" 
                      })
                      .replace(/\s/g, '')
                      .replace(/(\d{4})\.(\d{2})\.(\d{2})\./, '$1. $2. $3.')
                    : "정보 없음"}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default ScheduleList;