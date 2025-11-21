import React from "react";
import "./HelpModal.css";
import useMainStore from "../../../MainStore";

export default function HelpModal() {
  const { closeHelp } = useMainStore();

  return (
    <div className="help-overlay">
      <div className="help-modal">
        <h2 className="help-title">📘 실행 강제 시스템 웹 대시보드 도움말</h2>

        <div className="help-content">
          <p>
            이 대시보드에서는 실행 강제 시스템에서의 <strong>활동 요약</strong>,{" "}
            <strong>일정 관리</strong>, <strong>활동 피드백</strong> 등의 기능을 한 곳에서
            편리하게 사용할 수 있도록 제공합니다.
          </p>

        {/* 주요 기능 */}
        <div className="help-section">
            <h3 className="help-section-title">🔧 주요 기능</h3>
          <ul>
            <li>🏠 <strong>Overview</strong>에서 전체적인 요약 화면을 확인할 수 있습니다.</li>
            <li>📝 <strong>스케줄</strong>에서 일간/주간/월간별 스케줄을 확인하고 추가/삭제할 수 있습니다.</li>
            <li>📊 <strong>활동 요약</strong>에서 최근 작업 그래프를 확인하거나 활동 요약 보고서를 제공받을 수 있습니다.</li>
            <li>🚨 <strong>피드백</strong>에서 최근 세션별 작업에 대한 AI 기반 피드백을 제공받을 수 있습니다.</li>
            <li>⚙️ <strong>설정</strong>에서 필요한 환경 설정 등을 진행할 수 있습니다.</li>
          </ul>
        </div>

        {/* 활용 팁 및 예정된 업데이트*/}
        <div className="help-section">
            <h3 className="help-section-title">💡 활용 팁 및 예정된 업데이트</h3>
          <ul>
            <li>⏱️ 우측 상단 새로고침 버튼을 누르면 즉시 화면을 동기화할 수 있습니다.</li>
            <li>📝 추후 다크 모드 지원 추가 예정</li>
            <li>📝 추후 계정 연동 기능 추가 예정</li>
            <li>📝 추후 Overview 스케줄 견본 화면 추가 예정</li>
            <li>📝 추후 일정 수정 기능 추가 예정</li>
          </ul>
        </div>

        {/* 문의 및 개선 요청*/}
        <div className="help-section">
            <h3 className="help-section-title">📨 문의 및 개선 요청</h3>
          <p>
            사용 중 불편한 점이나 개선 요청이 있다면 언제든지 의견을 남겨주세요.
            더 나은 사용 경험을 위해 지속적으로 업데이트할 예정입니다.
            <br/>문의 : snake2010@inu.ac.kr
          </p>
        </div>
    </div>

        <button className="close-btn" onClick={closeHelp}>
          닫기
        </button>
      </div>
    </div>
  );
}