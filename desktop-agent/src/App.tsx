import { useState, useEffect } from "react";
// import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import LoginView from './components/LoginView.tsx';
import MainView from './components/MainView';


function App() {

  // // -------- Test 코드 -----------------
  // const [greetMsg, setGreetMsg] = useState("");
  // const [name, setName] = useState("");

  // async function greet() {
  //   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //   setGreetMsg(await invoke("greet", { name }));
  // }

  // // 에러 발생 시 UI에 표시하기 위한 상태
  // const [error, setError] = useState<string | null>(null);

  // // 모든 프로세스 요약 정보 관련 상태
  // const [processesSummary, setProcessesSummary] = useState<any[]>([]);
  // const [processesError, setProcessesError] = useState<string | null>(null);
  


  // // 1. 활성 창 정보 테스트 
  // useEffect(() => {
  //   invoke('get_current_active_window_info')
  //     .then((res) => console.log('Active Window Info:', res))
  //     .catch((err) => console.error('Error:', err));
  // }, []);

  // // 2. 모든 프로세스 요약 정보 테스트
  // useEffect(() => {
  //   invoke('get_all_processes_summary')
  //     .then((res) => {
  //       console.log('All Processes Summary:', res);
  //       setProcessesSummary(res as any);
  //     })
  //     .catch((err) => {
  //       console.error('Error getting all processes summary:', err);
  //       setProcessesError(err.toString());
  //     });
  // }, []);

  // // 3. 사용자 입력 빈도 통계 테스트 (주기적으로 업데이트)
  // useEffect(() => {
  //   console.log('--- Testing get_input_frequency_stats (every 2 seconds) ---');
  //   const intervalId = setInterval(() => {
  //     invoke('get_input_frequency_stats')
  //       .then((res) => {
  //         console.log('Input Frequency Stats:', res);
  //         setError(null); // 성공 시 에러 초기화
  //       })
  //       .catch((err) => {
  //         console.error('Error getting Input Frequency Stats:', err);
  //         setError(`입력 빈도 통계 에러: ${err}`);
  //       });
  //   }, 2000); // 2초마다 갱신

  //   return () => clearInterval(intervalId); // 컴포넌트 언마운트 시 인터벌 정리
  // }, []); // 컴포넌트 마운트 시 한 번만 실행


  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);

  const handleLoginSuccess = (): void => {
    setIsLoggedIn(true);
    console.log("Mock Login Success! Navigating to Main View.");
  };

  const handleLogout = (): void => {
    setIsLoggedIn(false);
    console.log("Mock Logout. Navigating to Login View.");
  };

  return (
    <div className="App">
      {isLoggedIn ? (
        <MainView onLogout={handleLogout} />
      ) : (
        <LoginView onLoginSuccess={handleLoginSuccess} />
      )}
    </div>
  );
}

export default App;
