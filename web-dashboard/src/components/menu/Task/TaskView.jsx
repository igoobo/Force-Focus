import React, { useState, useEffect, useRef } from 'react';
import './TaskView.css';
import useMainStore from '../../../MainStore.jsx';
import { useTaskStore } from "./TaskStore.jsx";
import { createPortal } from 'react-dom';

// 1. 대표 프로그램 프리셋 정의
const PROGRAM_PRESETS = [
  // 개발 / 코딩
  { name: 'VS Code', path: 'Code.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m10 16 1.5-1.5"/><path d="m14 8-1.5 1.5"/><path d="M15 6 9 18"/><path d="M16 18h4V6h-4"/><path d="M8 6H4v12h4"/></svg> },
  { name: 'IntelliJ', path: 'idea64.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M7 7h10"/><path d="M7 12h7"/><path d="M7 17h10"/></svg> },
  { name: 'PyCharm', path: 'pycharm64.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a10 10 0 1 0 10 10H12V2z"/><path d="M12 12 2.21 12"/><path d="M12 12 12 22"/><path d="m21.21 15.89-8.59-4.42"/></svg> },
  
  // 브라우저 / 자료 조사
  { name: 'Chrome', path: 'chrome.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="4"/><line x1="21.17" y1="8" x2="12" y2="8"/><line x1="3.95" y1="6.06" x2="8.54" y2="14"/><line x1="10.88" y1="21.94" x2="15.46" y2="14"/></svg> },
  { name: 'Edge', path: 'msedge.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M2 12c0-5.52 4.48-10 10-10s10 4.48 10 10-4.48 10-10 10a10 10 0 0 1-5.73-1.81"/><path d="M11.5 14a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z"/><path d="M14 11.5c1.5 0 3 .5 3 2s-1.5 2-3 2-3-1-3-2"/></svg> },
  { name: 'Firefox', path: 'firefox.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 22a10 10 0 0 0 8-16c-1.5-1-4-1-6 1s-2 6 2 8c2.5 1.5 3 4 1 6"/></svg> },

  // 문서 / 오피스
  { name: 'Word', path: 'winword.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M4 18V6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z"/><path d="m9 8 3 8 3-8"/></svg> },
  { name: 'Excel', path: 'excel.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 8h8v8H8z"/><path d="M3 10h18"/><path d="M10 3v18"/></svg> },
  { name: 'PowerPoint', path: 'powerpnt.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 16V8l5 4-5 4Z"/></svg> },

  // 노트 / 학습
  { name: 'Notion', path: 'notion.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6V3h18v3"/><path d="M5 3v18"/><path d="M19 3v18"/><path d="M7 6h10"/><path d="M7 12h10"/></svg> },
  { name: 'Obsidian', path: 'Obsidian.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m12 3-8 9 8 9 8-9-8-9Z"/><path d="M12 3v18"/><path d="m4 12 16 0"/></svg> },
  { name: 'OneNote', path: 'onenote.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 7v10l6-5-6-5Z"/></svg> },

  // 커뮤니케이션 / 화상회의
  { name: 'Slack', path: 'slack.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="13" y="2" width="3" height="8" rx="1.5"/><path d="M19 8.5a1.5 1.5 0 1 1 0 3h-4.5"/><rect x="8" y="14" width="3" height="8" rx="1.5"/><path d="M5 12.5a1.5 1.5 0 1 1 0-3h4.5"/></svg> },
  { name: 'Discord', path: 'discord.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/><path d="M7 12.8c0 0-1.2 2.2-1 4.5 1.3 1 3.2 1.5 5 1.5h2c1.8 0 3.7-.5 5-1.5.2-2.3-1-4.5-1-4.5"/></svg> },
  { name: 'Zoom', path: 'zoom.exe', icon: <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m22 8-6 4 6 4V8Z"/><rect width="14" height="12" x="2" y="6" rx="2"/></svg> },
];

export default function TaskView() {
  const { isDarkMode, isDirty, setIsDirty } = useMainStore();
  const fileInputRef = useRef(null); // 파일 선택창 제어를 위한 Ref

  // 스토어 연결
  const { tasks, loading, fetchTasks, addTask, updateTaskApps, deleteTask, setLocalPaths } = useTaskStore();
  
  // 모달 제어 상태
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isProgramModalOpen, setIsProgramModalOpen] = useState(false);
  const [popoverPos, setPopoverPos] = useState({ top: 0, left: 0, isBottom: true });
  const [newSessionName, setNewSessionName] = useState(''); // 입력 필드 상태
  const [activeSelection, setActiveSelection] = useState({ sessionId: null, index: null }); // 현재 수정 중인 위치 추적

  // 초기 데이터 로드 (컴포넌트 마운트 시)
  useEffect(() => {
    fetchTasks();
  }, []);

  const markAsDirty = () => { if (!isDirty) setIsDirty(true); };

  // 파일 탐색기 열기
  const triggerFilePicker = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  // 탐색기에서 파일 선택 시 처리
  const handleFileChange = (e) => {
    const file = e.target.files[0];
    if (file) {
      handleSelectProgram(file.name); // 파일명만 추출하여 입력
      e.target.value = ''; // 같은 파일을 다시 선택할 수 있도록 초기화
    }
  };

  const handleOpenProgramModal = (e, sessionId, index) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const popoverWidth = 400;
    const popoverHeight = 340;
    const margin = 10;

    let left = rect.left + (rect.width / 2) - (popoverWidth / 2) - 70;
    if (left < margin) left = margin;
    if (left + popoverWidth > window.innerWidth - margin) {
      left = window.innerWidth - popoverWidth - margin;
    }

    const spaceBelow = window.innerHeight - rect.bottom;
    let top;
    let isBottom = true;

    if (spaceBelow < popoverHeight && rect.top > popoverHeight) {
      top = rect.top - popoverHeight + 20;
      isBottom = false;
    } else {
      top = rect.bottom + 10;
      isBottom = true;
    } 

    setPopoverPos({ top, left, isBottom });
    setActiveSelection({ sessionId, index });
    setIsProgramModalOpen(true);
  };

  const handleSelectProgram = (path) => {
    if (!path.trim()) return;
    const { taskId, index } = activeSelection;
    const task = tasks.find(t => t.id === taskId);
    const newPaths = [...(task.appPaths || [])];
    newPaths[index] = path;
    
    setLocalPaths(taskId, newPaths);
    markAsDirty();
    setIsProgramModalOpen(false);
  };

  const handleSave = async () => {
    const hasEmptyPath = tasks.some(task => 
      task.appPaths.some(path => path.trim() === "")
    );
    if (hasEmptyPath) {
      alert("입력되지 않은 프로그램이 있습니다. 모든 빈 칸을 완성해 주세요.");
      return;
    }

    try {
      // 모든 작업의 변경사항을 병렬로 서버에 전송
      await Promise.all(tasks.map(t => updateTaskApps(t.id, t.appPaths)));
      alert("모든 설정이 서버에 성공적으로 저장되었습니다.");
      setIsDirty(false);
    } catch (err) {
      alert("일부 데이터를 저장하는 데 실패했습니다.");
    }
  };

  const addPathInput = (taskId) => {
    const task = tasks.find(t => t.id === taskId);
    if (task.appPaths.length >= 5) return alert("작업당 최대 5개까지만 가능합니다.");
    setLocalPaths(taskId, [...task.appPaths, '']);
    markAsDirty();
  };

  const removePathInput = (taskId, index) => {
    const task = tasks.find(t => t.id === taskId);
    const newPaths = task.appPaths.filter((_, i) => i !== index);
    setLocalPaths(taskId, newPaths);
    markAsDirty();
  };

  const handleAddSession = async () => {
    if (!newSessionName.trim()) return;
    await addTask(newSessionName);
    setNewSessionName('');
    setIsModalOpen(false);
    setIsDirty(false);
  };

  const handleDeleteSession = async (id) => {
    if (window.confirm("이 작업 유형을 삭제하시겠습니까? (이 작업과 연결된 일정들이 영향을 받을 수 있습니다)")) {
      await deleteTask(id);
      setIsDirty(false);
    }
  };

  useEffect(() => {
    const handleBeforeUnload = (e) => {
      if (isDirty) {
        e.preventDefault();
        e.returnValue = ""; 
      }
    };
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [isDirty]);

  useEffect(() => {
    const handleScrollClose = () => {
      if (isProgramModalOpen) setIsProgramModalOpen(false);
    };

    if (isProgramModalOpen) {
      window.addEventListener('scroll', handleScrollClose, { passive: true });
    }
    return () => window.removeEventListener('scroll', handleScrollClose);
  }, [isProgramModalOpen]);

  if (loading && tasks.length === 0) return <div className="loading-area">데이터를 불러오는 중...</div>;

  return (
    <div className={`task-container ${isDarkMode ? 'dark-theme' : ''}`}>
      {/* 숨겨진 파일 인풋 필드 */}
      <input 
        type="file" 
        ref={fileInputRef} 
        style={{ display: 'none' }} 
        accept=".exe"
        onChange={handleFileChange} 
      />

      <header className="task-header">
        <div className="header-text">
          <h2>🛠️ 작업 설정</h2>
          <p className="task-description">
             작업 설정에서는 작업별 강제 실행 프로그램 지정을 통해, 세션 시작 시 자동으로 프로그램을 실행 및 통제할 수 있습니다. (작업별 최대 5개까지 지정 가능)
          </p>
        </div>
        <div className="header-actions">
          <button className="add-task-btn" onClick={() => setIsModalOpen(true)}>+ 새 작업 추가</button>
          <button className="save-db-btn" onClick={handleSave}>저장하기</button>
        </div>
      </header>

      <div className="task-grid">
        {tasks.map((task) => (
          <div key={task.id} className="session-card">
            <div className="session-card-header">
              <div className="session-info">
                <span className={`session-dot ${task.isCustom ? 'custom' : 'default'}`}></span>
                {task.label}
              </div>
              {/* 사용자가 추가한 작업(isCustom: true)인 경우에만 삭제 버튼 노출 */}
              {task.isCustom && (
                <button className="delete-session-btn" onClick={() => handleDeleteSession(task.id)}>
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
                </button>
              )}
            </div>
            
            <div className="card-divider"></div>

            <div className="path-input-list">
              <label className="section-label">실행 앱 리스트</label>
              {task.appPaths?.length > 0 ? (
                <div className="scrollable-path-area">
                  {task.appPaths.map((path, idx) => (
                    <div key={idx} className="path-input-row">
                      <div className="input-wrapper">
                        <input 
                          type="text" 
                          value={path} 
                          readOnly 
                          placeholder="프로그램 선택"
                          onClick={(e) => handleOpenProgramModal(e, task.id, idx)}
                          style={{ cursor: 'pointer' }}
                        />
                        <button className="inline-browse-btn" onClick={(e) => handleOpenProgramModal(e, task.id, idx)}>
                          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="1"></circle><circle cx="19" cy="12" r="1"></circle><circle cx="5" cy="12" r="1"></circle></svg>
                        </button>
                      </div>
                      <button className="remove-path-btn-styled" onClick={() => removePathInput(task.id, idx)}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
                      </button>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="empty-path-message">등록된 프로그램이 없습니다.</div>
              )}
            </div>

            <button 
              className="add-path-row-btn" 
              onClick={() => addPathInput(task.id)}
              disabled={task.appPaths.length >= 5}
            >
              {task.appPaths.length >= 5 ? "한도 초과 (최대 5개)" : "+ 프로그램 추가"}
            </button>
          </div>
        ))}
      </div>

      {/* 새 작업 추가 모달 */}
      {isModalOpen && createPortal(
        <div className={`modal-overlay ${isDarkMode ? 'dark-theme' : ''}`}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>새 작업 추가</h3>
              <p>작업 유형 명칭을 입력하세요.</p>
            </div>
            <div className="modal-body">
              <div className="path-input-group">
                <input 
                  autoFocus
                  type="text" 
                  value={newSessionName} 
                  onChange={(e) => setNewSessionName(e.target.value)}
                  placeholder="예: 영상 편집"
                />
              </div>
            </div>
            <div className="modal-footer">
              <button className="modal-cancel-btn" onClick={() => setIsModalOpen(false)}>취소</button>
              <button className="modal-confirm-btn" onClick={handleAddSession}>추가하기</button>
            </div>
          </div>
        </div>,
        document.body
      )}

      {/* 프로그램 선택 팝오버 */}
      {isProgramModalOpen && createPortal(
        <div className={`popover-overlay ${isDarkMode ? 'dark-theme' : ''}`} onClick={() => setIsProgramModalOpen(false)}>
          <div 
            className="program-popover" 
            style={{ 
              top: popoverPos.top, 
              left: popoverPos.left,
              transformOrigin: popoverPos.isBottom ? 'top center' : 'bottom center' 
            }} 
            onClick={(e) => e.stopPropagation()}
          >
            <div className="popover-body">
              <div className="mini-program-grid">
                {PROGRAM_PRESETS.map((prog) => (
                  <button key={prog.name} className="mini-prog-item" onClick={() => handleSelectProgram(prog.path)}>
                    <span className="mini-icon">{prog.icon}</span>
                    <span className="mini-name">{prog.name}</span>
                  </button>
                ))}
                
                {/* 직접 입력 대신 파일 찾기 버튼 */}
                <button className="mini-prog-item add-custom-mini" onClick={triggerFilePicker}>
                  <span className="mini-icon">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                  </span>
                  <span className="mini-name">파일 찾기</span>
                </button>
              </div>
            </div>
          </div>
        </div>, 
        document.body
      )}
    </div>
  );
}