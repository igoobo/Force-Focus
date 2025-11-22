import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

// Rust에서 오는 페이로드 타입
type InterventionPayload = "notification" | "overlay";

/**
 * '가벼운 개입(Notification)' 이벤트만 처리하는 커스텀 훅
 */
export function useInterventionListener() {
  
  // 백엔드 통신 에러 상태
  const [backendError, setBackendError] = useState<string | null>(null);

  /**
   * OS 레벨의 알림을 전송하는 함수
   */
  const sendOsNotification = async () => {
    try {
      // 1. 권한 확인
      let permissionGranted = await isPermissionGranted();
      
      // 2. 권한이 없으면 요청
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }

      // 3. 권한이 있으면 알림 전송
      if (permissionGranted) {
        sendNotification({
          title: '집중할 시간입니다!',
          body: '현재 활동이 "딴짓"으로 감지되었습니다.',
          // (추가) 아이콘 등 설정 가능
        });
      } else {
        console.warn('OS notification permission denied.');
      }
    } catch (e) {
      console.error("Failed to send OS notification:", e);
      setBackendError(`OS 알림 전송 실패: ${e}`);
    }
  };

  // Rust 이벤트 리스너 설정
  useEffect(() => {
    console.log("Setting up Rust event listener...");
    let unlistenFn: (() => void) | null = null;

    const setupListener = async () => {
      try {
        const unlisten = await listen<InterventionPayload>("intervention-trigger", (event) => {
          console.log(`Rust Event Received: ${event.payload}`);
          
          if (event.payload === "overlay") {
            // 오버레이는 Rust가 처리하므로 무시
          } else if (event.payload === "notification") {
            // "약한 개입" 시 OS 알림 전송
            sendOsNotification();
          }
        });
        unlistenFn = unlisten;
      } catch (e) {
        console.error("Failed to setup Rust listener:", e);
        setBackendError(`이벤트 리스너 설정 실패: ${e}`);
      }
    };

    setupListener();

    return () => {
      console.log("Cleaning up Rust event listener...");
      if (unlistenFn) unlistenFn();
    };
  }, []); // 마운트 시 1회 실행

  return { backendError };
}