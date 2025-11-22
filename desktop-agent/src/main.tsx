import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// // 파일 위치: Force-Focus/desktop-agent/src/main.tsx
// import React from 'react';
// import ReactDOM from 'react-dom/client';
// import App from './App';
// // import './index.css';

// // MSW 활성화 코드 시작
// async function enableMocking() {
//   // 개발 환경(NODE_ENV가 'development')이 아닐 때는 Mocking을 비활성화합니다.
//   if (process.env.NODE_ENV !== 'development') {
//     return;
//   }
 
//   // './mocks/browser' 모듈을 동적으로 임포트하여 MSW worker를 가져옵니다.
//   const { worker } = await import('./mocks/browser');
  
//   console.log('MSW trying to enable...');
//   // 서비스 워커를 시작합니다.
//   // 'onUnhandledRequest: "bypass"' 옵션은
//   // Mocking하지 않은 API 요청은 실제 네트워크로 보내도록 합니다.
//   return worker.start({
//     onUnhandledRequest: 'bypass',
//   });
// }
// // MSW 활성화 코드 끝

// // enableMocking 함수를 먼저 실행하고, Mocking 설정이 완료되면 App을 렌더링합니다.
// enableMocking().then(() => {
//   ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
//     <React.StrictMode>
//       <App />
//     </React.StrictMode>,
//   );
// });