import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

import LoginView from './components/LoginView.tsx';
import MainView from './components/MainView';
import InterventionOverlay from './components/InterventionOverlay';

// // --- Backend Communicator 테스트를 위한 함수 및 상태 ---
// function BackendTestComponent() {
//   const [feedbackTestResult, setFeedbackTestResult] = useState<string>("(대기 중)");

//   // 'submit_feedback' invoke를 테스트하는 함수
//   async function testSubmitFeedback() {
//     console.log("Testing 'submit_feedback' invoke...");
//     setFeedbackTestResult("Sending request...");
//     try {
//       // Rust의 submit_feedback(feedback_type: String, ...) 커맨드를 호출
//       await invoke('submit_feedback', {
//         feedbackType: 'is_work_from_test_button' // Rust 함수 인수에 매핑
//       });
      
//       const successMsg = "✅ Success: Feedback submitted!";
//       console.log(successMsg);
//       setFeedbackTestResult(successMsg);
      
//     } catch (error) {
//       const errorMsg = `❌ Error: ${error}`;
//       console.error("Feedback test error:", error);
//       setFeedbackTestResult(errorMsg);
//     }
//   }

//   return (
//     <div style={{ 
//       padding: '10px', 
//       backgroundColor: '#2a2a2a', 
//       color: 'white', 
//       borderBottom: '1px solid #555' 
//     }}>
//       <h3 style={{ marginTop: 0 }}>[Backend Communicator Test Panel]</h3>
//       <button onClick={testSubmitFeedback}>
//         Test Invoke: 'submit_feedback("is_work")'
//       </button>
//       <p style={{ fontSize: '0.9em', wordBreak: 'break-all' }}>
//         Test Result: {feedbackTestResult}
//       </p>
//     </div>
//   );
// }
// // --- 테스트 컴포넌트 종료 ---

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


  // 개입 오버레이의 표시 여부를 관리하는 상태
  const [showOverlay, setShowOverlay] = useState<boolean>(false);
  // 백엔드 에러 메시지 표시용
  const [backendError, setBackendError] = useState<string | null>(null);


  // Rust 백엔드로부터 "intervention-trigger" 이벤트를 수신(listen)
  useEffect(() => {
    console.log("Setting up Rust event listener...");

    // 이벤트 페이로드(payload)의 타입을 정의 (여기서는 "notification" 또는 "overlay")
    type InterventionPayload = "notification" | "overlay";

    const unlisten = listen<InterventionPayload>("intervention-trigger", (event) => {
      console.log(`Rust Event Received: ${event.payload}`);
      
      // Rust가 "overlay" 명령을 보냈을 때만 오버레이
      if (event.payload === "overlay") {
        setShowOverlay(true);
      }
      // (추후 "notification" 페이로드에 대한 OS 알림 로직도 추가 가능)
    });

    // 컴포넌트가 언마운트될 때 리스너를 정리
    return () => {
      console.log("Cleaning up Rust event listener...");
      unlisten.then(f => f());
    };
  }, []); // 컴포넌트 마운트 시 한 번만 실행


  // "이건 업무임" 버튼 클릭 시 실행될 핸들러
  const handleInterventionFeedback = async () => {
    console.log("Feedback button clicked. Invoking 'submit_feedback'...");
    try {
      // Rust의 submit_feedback 커맨드를 호출
      await invoke('submit_feedback', {
        feedbackType: 'is_work', // 피드백 타입을 전송
      });
      console.log("Feedback submitted successfully.");
      setBackendError(null); // 성공 시 에러 메시지 제거
    } catch (error) {
      console.error("Failed to submit feedback:", error);
      setBackendError(`피드백 전송 실패: ${error}`);
    } finally {
      // 피드백을 전송했으므로 오버레이 닫기
      setShowOverlay(false);
    }
  };

  // "닫기" 버튼 클릭 시 실행될 핸들러
  const handleCloseOverlay = () => {
    console.log("Close button clicked. Hiding overlay.");
    setShowOverlay(false);
  };

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

      {/* 테스트용 컴포넌트를 앱 최상단에 렌더링 */}
      {/* <BackendTestComponent /> */}

      {/* 백엔드 에러가 발생하면 화면 상단에 표시 */}
      {backendError && (
        <div style={{ backgroundColor: 'red', color: 'white', padding: '10px', textAlign: 'center' }}>
          {backendError}
        </div>
      )}

      {isLoggedIn ? (
        <MainView onLogout={handleLogout} />
      ) : (
        <LoginView onLoginSuccess={handleLoginSuccess} />
      )}

      {/* showOverlay 상태가 true일 때만 InterventionOverlay를 렌더링 */}
      {showOverlay && (
        <InterventionOverlay 
          onFeedback={handleInterventionFeedback}
          onClose={handleCloseOverlay}
        />
      )}
    </div>
  );
}

export default App;
