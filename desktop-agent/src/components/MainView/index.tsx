// 파일 위치: Force-Focus/desktop-agent/src/components/MainView/index.tsx

import { FC, useState, useEffect, useCallback } from 'react';
import { fetchCurrentSession, fetchTasks, startSession, endSession, fetchTaskById } from '../../api'; // API 함수 임포트
import { Session, Task } from '../../types';

// 하위 컴포넌트 임포트
import HeaderControls from './HeaderControls';
import SessionDisplay from './SessionDisplay';
import MainControls from './MainControls';
import FooterControls from './FooterControls';

interface MainViewProps {
  onLogout: () => void;
}

const MainView: FC<MainViewProps> = ({ onLogout }) => {
    const [currentSession, setCurrentSession] = useState<Session | null>(null);
    const [currentTask, setCurrentTask] = useState<Task | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);
    const [syncStatus, setSyncStatus] = useState<"online" | "offline" | "syncing">("online"); // 동기화 상태 Mock

    // 세션 및 Task 데이터를 불러오는 함수
    const loadSessionAndTask = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            const session = await fetchCurrentSession(); // MSW Mock API 호출
            setCurrentSession(session);

            if (session && session.task_id) {
            // 특정 task_id를 가진 Task만 가져오도록 수정
            const task = await fetchTaskById(session.task_id);
            setCurrentTask(task);
            } else {
            setCurrentTask(null);
            }
            setLoading(false);
        } catch (err: any) {
            console.error("Error loading session/task:", err);
            setError(err.message || "Failed to load session data.");
            setCurrentSession(null);
            setCurrentTask(null);
            setLoading(false);
            setSyncStatus("offline"); // 에러 발생 시 오프라인으로 간주
        }
    }, []); // 의존성 배열 비움: 컴포넌트 마운트 시 한 번만 생성

    useEffect(() => {
        loadSessionAndTask();

        // (선택) 10초마다 동기화 상태 토글 Mock
        const syncInterval = setInterval(() => {
            setSyncStatus(prev => {
            if (prev === "online") return "syncing";
            if (prev === "syncing") return "online";
            return "online"; // 기본
            });
        }, 10000);

        return () => clearInterval(syncInterval); // 컴포넌트 언마운트 시 클린업
    }, [loadSessionAndTask]); // loadSessionAndTask가 변경될 때마다 useEffect 재실행 (useCallback 덕분에 최초 1회)

    // 세션 시작/종료를 토글하는 함수
    const handleToggleSession = async () => {
        try {
            if (currentSession?.status === 'active') {
            // 세션 종료
            if (currentSession.id) {
                await endSession(currentSession.id); // MSW Mock API 호출
                setCurrentSession(prev => prev ? { ...prev, status: 'ended' } : null);
                console.log("Session ended (Mock).");
            }
            } else {
            // 세션 시작 (예: 'task-coding-session'으로)
            // handlers.ts의 mockTasks 중 하나를 선택
            const defaultTaskId = 'task-coding-session'; // 기본 Task ID
            const newSession = await startSession(defaultTaskId, 60); // MSW Mock API 호출
            setCurrentSession(newSession);
            const task = await fetchTaskById(defaultTaskId); // 새로 시작된 세션의 Task 정보 가져오기
            setCurrentTask(task);
            console.log("Session started (Mock):", newSession);
            }
            // 세션 상태 변경 후 데이터를 다시 불러와 최신화
            await loadSessionAndTask();
        } catch (err: any) {
            console.error("Error toggling session:", err);
            setError(err.message || "Failed to toggle session.");
        }
    };

    // 설정 화면 열기 핸들러 (미구현)
    const handleOpenSettings = () => {
        console.log("Settings opened (not implemented yet).");
        // 실제 구현 시 설정 화면으로 이동 로직
    };

    // 웹 대시보드로 이동 핸들러 (미구현)
    const handleGoToDashboard = () => {
        console.log("Go to Dashboard (not implemented yet).");
        // 실제 구현 시 웹 대시보드 URL 열기 로직
    };

    if (loading) {
        return (
            <div className="flex items-center justify-center min-h-screen bg-gray-800 text-white">
                <p>데이터 로딩 중...</p>
            </div>
        );
    }

    if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-800 text-white">
        <p className="text-red-500">에러 발생: {error}</p>
      </div>
    );
    }

    return(
        <div className="flex flex-col items-center justify-between min-h-screen bg-gray-800 text-white p-6">
            {/* HeaderControls 컴포넌트에 Props 전달 */}
            <HeaderControls
                syncStatus={syncStatus}
                onOpenSettings={handleOpenSettings}
                onGoToDashboard={handleGoToDashboard}
            />
            {/* SessionDisplay 컴포넌트에 Props 전달 */}
            <SessionDisplay session={currentSession} task={currentTask} />
            {/* MainControls 컴포넌트에 Props 전달 */}
            <MainControls
                sessionStatus={currentSession?.status || null}
                onToggleSession={handleToggleSession}
            />
            {/* FooterControls 컴포넌트에 Props 전달 */}
            <FooterControls onLogout={onLogout} />
        </div>

    );
};

export default MainView;