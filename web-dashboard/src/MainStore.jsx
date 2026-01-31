import { create } from 'zustand';
import { persist } from "zustand/middleware";

const useMainStore = create(
  persist(
    (set) => ({
      // 1. 인증 상태
      isLoggedIn: !!localStorage.getItem('accessToken'),
      
      // 2. 로그아웃 액션
      logout: () => {
        localStorage.removeItem('accessToken');
        localStorage.removeItem('refreshToken');
        localStorage.removeItem('userEmail');
        set({ isLoggedIn: false, activeMenu: 'Overview' });
      },

      // 3. 로그인 액션
      login: () => set({ isLoggedIn: true }),

      // 4. 메뉴 열림/닫힘 상태 (각 메뉴)
      isOpen: true,
      toggleMenu: () => set((state) => ({ isOpen: !state.isOpen })),

      // 5. 현재 활성화된 메뉴 (전체 메뉴 목록)
      activeMenu: 'Overview',
      scheduleViewMode: "week",

      setScheduleViewMode: (mode) => set({ scheduleViewMode: mode }),
      setActiveMenu: (menu, initialView = null) => {
        if (initialView) {
          set({ activeMenu: menu, scheduleViewMode: initialView });
        } else {
          set({ activeMenu: menu });
        }
      },

      // 6. 스케줄 메뉴 진입 시 적용할 임시 뷰 모드 (스케줄 메뉴)
      scheduleInitialView: null, 
      clearScheduleInitialView: () => set({ scheduleInitialView: null }),

      // 7. 활동 요약 메뉴 진입 시 적용할 뷰 모드 (활동 요약 메뉴)
      activityViewMode: 'horizontal', 
      setActivityViewMode: (mode) => set({ activityViewMode: mode }),

      // 8. 피드백 메뉴 진입 시 적용할 뷰 모드 (피드백 메뉴)
      feedbackViewMode: '종합', 
      setFeedbackViewMode: (mode) => set({ feedbackViewMode: mode }),

      // 9. 설정 메뉴 진입 시 적용할 뷰 모드 (설정 메뉴)
      settingsViewMode: 'limit', 
      setSettingsViewMode: (mode) => set({ settingsViewMode: mode }),
      
      // 10. 작업 변경 사항 추적 (작업 메뉴)
      isDirty: false,
      setIsDirty: (status) => set({ isDirty: status }),

      // 11. 도움말 모달 열림/닫힘 상태 (도움말)
      isHelpOpen: false,
      openHelp: () => set({ isHelpOpen: true }),
      closeHelp: () => set({ isHelpOpen: false }),

      // 12. 다크모드 상태 및 토글 함수 (다크 모드)
      isDarkMode: false,
      toggleDarkMode: () => set((state) => ({ isDarkMode: !state.isDarkMode })),
    }),
    { name: 'main-storage',
      partialize: (state) => ({ isDarkMode: state.isDarkMode, isLoggedIn: state.isLoggedIn })
    }
  )
);

export default useMainStore;