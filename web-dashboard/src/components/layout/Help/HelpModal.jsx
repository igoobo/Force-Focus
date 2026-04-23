import React, { useState, useEffect } from 'react';
import './HelpModal.css';
import useMainStore from '../../../MainStore';

// 도움말 콘텐츠 데이터 구조화
const helpContent = {
  all: {
    label: '소개',
    subs: {
      intro: { 
        label: '소개', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">Force-Focus Web Dashboard에 오신 것을 환영합니다!</h4>
            <p className="guide-text">
              이 대시보드는 Force-Focus 시스템 기반의 웹 대시보드입니다.<br/>
              사용자의 작업 스케줄을 등록 및 관리하고, 작업 환경을 통제하기 위한 프로그램 지정, 수집된 활동 데이터를 기반으로 한 활동 요약 및 피드백을 제공합니다.
            </p>
            
            <div className="guide-card-grid">
              <div className="guide-card">
                <span className="guide-card-icon">🎯</span>
                <h5>작업 환경 통제</h5>
                <p>자신의 작업 환경을 통제하기 위한 실행 강제 프로그램을 선택할 수 있습니다.</p>
              </div>
              <div className="guide-card">
                <span className="guide-card-icon">📊</span>
                <h5>활동 요약 및 피드백 제공</h5>
                <p>수집된 활동 데이터를 통해 나의 활동 요약과 AI 기반 피드백을 확인하고, 본인의 업무 패턴을 분석 및 개선할 수 있습니다.</p>
              </div>
            </div>

            <div className="guide-info-box">
              <p>💡 <strong>참고:</strong> 모든 데이터는 실시간으로 동기화되며, 상단 바의 [새로고침] 버튼을 통해 최신 상태를 유지할 수 있습니다. </p>
            </div>
          </div>
        ) 
      },
      menus: {
        label: '주요 메뉴 설명', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">대시보드 핵심 메뉴 가이드</h4>
            <p className="guide-text">각 메뉴는 사용자의 Force-Focus 시스템 사용에 도움이 되는 고유한 기능을 제공합니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>메뉴명</th>
                  <th>주요 기능 및 역할</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><div className="layout-tag">Overview</div></td>
                  <td>할 일 목록, 최근 활동 요약, 최근 활동 그래프, 최근 세션 피드백 등을 한 눈에 확인할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><div className="layout-tag">스케줄</div></td>
                  <td>일간/주간/월간 단위로 일정을 확인하거나, 일정을 추가/수정/삭제할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><div className="layout-tag">작업</div></td>
                  <td>각 작업 종류별로 강제 실행할 프로그램 목록을 등록하고 관리할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><div className="layout-tag">활동 요약</div></td>
                  <td>최근 활동을 요약하는 그래프와 보고서를 통해 최근 7일간의 활동 내역을 요약하여 분석할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><div className="layout-tag">피드백</div></td>
                  <td>종료된 세션에 대한 AI 기반 분석 결과를 확인하고 피드백을 통하여 개선점을 제공받을 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        ) 
      },
    }
  },
  overview: {
    label: 'Overview',
    subs: {
      intro: { 
        label: '화면 구성', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">Overview: 종합 대시보드 구성 요소</h4>
            <p className="guide-text">초기 화면의 각 카드는 상세 메뉴의 핵심 정보를 요약하여 보여줍니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>항목</th>
                  <th>상세 설명 및 연동 기능</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">시간표</span></td>
                  <td>스케줄 메뉴와 동일하게 일/주/월 단위로 전환 가능하며, 현재 시간에 맞춰 자동으로 스크롤이 이동합니다. <br/> 해당 카드 클릭 시 바로 <b>스케줄</b> 메뉴로 이동합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">최근 활동 요약</span></td>
                  <td>최근 7일간의 데이터를 1시간 단위로 자동 갱신하며, 주요 사용 앱과 사용 시간 등을 요약하여 제공합니다. <br/> 해당 카드 클릭 시 바로 <b>활동 요약</b> 메뉴로 이동합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">최근 작업 피드백</span></td>
                  <td>가장 최근에 완료된 작업 세션의 AI 기반 피드백을 자동으로 불러와 요약 정보를 제공합니다. <br/> 해당 카드 클릭 시 바로 <b>피드백</b> 메뉴의 최근 세션으로 이동합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">최근 활동 그래프</span></td>
                  <td>최근 7일간의 활동 데이터에 대한 그래프 정보를 제공합니다. <br/> 해당 카드 클릭 시 바로 <b>활동 요약</b> 메뉴로 이동합니다.</td>
                </tr>
              </tbody>
            </table>

            <div className="guide-info-box">
              <p>💡 <strong>참고:</strong> 각 표의 항목을 클릭하면 해당 상세 페이지로 즉시 이동할 수 있습니다.</p>
            </div>
          </div>
        ) 
      },
    }
  },
  schedule: {
    label: '스케줄',
    subs: {
      overall: { 
        label: '종합', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">스케줄 메뉴: 통합 일정 관리</h4>
            <p className="guide-text">사용자의 일정을 데이터베이스와 실시간으로 동기화하여 관리합니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">일정 조회</span></td>
                  <td><b>일간/주간/월간 스케줄 메뉴</b>를 각각 활용하여 일정을 다양한 시간 단위로 조회할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 추가</span></td>
                  <td>우측 상단의 <b>[+ 일정 추가]</b> 버튼을 클릭하여 일정 추가를 수행할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 수정</span></td>
                  <td>각 메뉴의 화면에 표시된 일정 박스를 직접 클릭하면 해당 일정에 대한 <b>수정 화면</b>을 즉시 확인할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 삭제</span></td>
                  <td><b>[− 일정 삭제]</b> 버튼을 통해 관리 모달을 열어 사용하지 않는 일정을 제거할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 목록</span></td>
                  <td>현재까지 추가한 일정 정보를 <b>카드 형태의 목록</b>으로 한눈에 확인할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      day: { 
        label: '일간 스케줄', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">일간(Day) 뷰: 정밀 관리</h4>
            <p className="guide-text">특정 날짜의 일간 스케줄을 24시간 기준으로 확인할 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">일별 스케줄 조회</span></td>
                  <td>기본적으로 <b>오늘 날짜</b>의 스케줄을 즉시 조회할 수 있으며, 상단에서 날짜를 변경하여 해당 날짜의 일간 스케줄을 조회할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 즉시 수정</span></td>
                  <td>타임라인에 포함된 일정을 클릭할 시 일정을 <b>즉시 수정</b>할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      week: { 
        label: '주간 스케줄', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">주간(Week) 뷰: 흐름 파악</h4>
            <p className="guide-text">한 주간의 모든 일정을 요일별로 배치하여 전체적인 작업 분포를 시각화합니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">주간별 스케줄 조회</span></td>
                  <td>주간 단위로 예정된 일정과 시간을 각 요일별로 한눈에 파악할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 즉시 수정</span></td>
                  <td>주간 뷰에서 포함된 각각의 일정을 클릭할 시 일정을 <b>즉시 수정</b>할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      month: { 
        label: '월간 스케줄', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">월간(Month) 뷰: 장기 계획</h4>
            <p className="guide-text">달력 형태의 인터페이스를 통해 한 달의 일정을 확인 가능하며, 특정 날짜 선택을 통해 일간 스케줄로도 빠르게 이동 가능합니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">월간별 스케줄 조회</span></td>
                  <td>월간 단위로 예정된 일정과 시간을 각 날짜별로 한눈에 파악할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">빠른 일간 뷰 연동</span></td>
                  <td>특정 날짜 칸을 클릭하면 해당 날짜에 해당하는 <b>일간 뷰</b>를 즉시 확인할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">일정 오버레이</span></td>
                  <td>각 날짜에 등록된 핵심 일정들이 요약 표시되며, 클릭 시 <b>상세 수정</b>이 가능합니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      list: { 
        label: '일정 목록', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">일정 목록: 텍스트 뷰</h4>
            <p className="guide-text">기존에 추가된 전체 일정을 카드 모양의 목록 형태로 나열하여 관리의 편의성을 제공하며, CSV 파일로 현재 등록된 일정 정보를 내보낼 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">전체 일정 조회</span></td>
                  <td>등록된 모든 일정을 최신순으로 표시하며, 전체 일정의 상세 정보를 일괄적으로 확인할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">CSV 내보내기</span></td>
                  <td>우측 상단의 <b>CSV로 내보내기</b> 버튼을 클릭하여 현재 전체 일정 데이터를 CSV 파일로 내려받을 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      add: { 
        label: '일정 추가', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">신규 일정 등록</h4>
            <p className="guide-text">개별 등록 방식과 CSV 파일을 이용한 일괄 등록 방식을 지원합니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>방식</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">개별 등록</span></td>
                  <td>일정 이름, 작업 유형, 시간 정보, 설명 등의 내용을 <b>사용자가 직접 입력</b>하여 개별적으로 등록합니다. </td>
                </tr>
                <tr>
                  <td><span className="layout-tag">CSV 일괄 등록</span></td>
                  <td>
                    좌측 하단의 <b>[CSV 일괄 등록]</b> 버튼을 클릭하여 CSV 파일을 업로드하면 <b>다수의 일정을 한 번에</b> 업로드할 수 있습니다.
                    <br />CSV 파일은 <b>반드시 형식에 맞게</b> 작성 및 업로드되어야 하며, 형식에 맞지 않는 파일 업로드 시 시스템에 업로드되지 않을 수 있습니다.
                  </td>
                </tr>
              </tbody>
            </table>
              <div className="guide-info-box" style={{ marginTop: '10px' }}>
                <strong>일정 추가 시 CSV 파일에 필요한 요소</strong> <br></br><code>name, task_name, description, start_date, start_time, end_date, end_time</code><br/>
                <small>* task_name은 시스템에 등록된 작업 유형의 이름과 일치해야 하며, 날짜(YYYY-MM-DD)와 시간(HH:MM) 형식을 반드시 준수해야 합니다.</small>
              </div>
          </div>
        ) 
      },
      edit: { 
        label: '일정 수정', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">일정 정보 수정</h4>
            <p className="guide-text">기존에 등록된 일정의 세부 정보를 변경할 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>단계</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">수정 모달 진입</span></td>
                  <td>일정 목록이나 각 스케줄 화면에서 각 일정을 클릭하면 <b>상세 수정 모달</b>이 나타납니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">데이터 저장</span></td>
                  <td>내용을 수정한 후 저장 버튼을 누르면 실시간으로 데이터를 저장할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        ) 
      },
      delete: { 
        label: '일정 삭제', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">일정 영구 삭제</h4>
            <p className="guide-text">더 이상 수행하지 않거나 잘못 등록된 일정을 삭제합니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>구분</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">삭제 처리</span></td>
                  <td>스케줄 화면의 우측 상단 <b>삭제</b> 버튼을 클릭하여 삭제를 진행할 수 있습니다. <br></br>단, 삭제된 데이터는 복구할 수 없으므로 신중히 결정해 주세요.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">데이터 연동</span></td>
                  <td>일정을 삭제하더라도 해당 일정에 연결되었던 <b>작업(Task)</b> 유형 정보는 삭제되지 않습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        ) 
      },
    }
  },
  task: {
    label: '작업',
    subs: {
      overall: { 
        label: '종합', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">작업 메뉴: 집중력 향상을 위한 환경 통제</h4>
            <p className="guide-text">
              특정 작업 카테고리를 실행할 때, 오직 지정된 프로그램만 사용할 수 있도록 강제하는 <b>실행 강제 프로그램 목록</b>을 설정합니다.
            </p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">카테고리 매칭</span></td>
                  <td>개발, 디자인, 문서 작성 등 해당되는 작업 성격에 맞는 <b>대표 실행 프로그램</b>을 각각 지정할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">강제 실행 관리</span></td>
                  <td>설정된 프로그램은 세션 시작 시 시스템에 의해 강제 실행되며, 작업 몰입을 방해하는 다른 요소의 접근을 제한합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">실시간 동기화</span></td>
                  <td>작업 카테고리 및 프로그램 목록 수정 후 저장 시 변경 사항은 즉시 서버에 반영되어, 다음번 작업 세션부터 바로 적용됩니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      program_setting: { 
        label: '프로그램 설정', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">프로그램 등록 및 변경 방법</h4>
            <p className="guide-text">시스템에서 기본적으로 제공하는 프리셋을 사용하거나, 본인이 사용하는 프로그램의 경로를 직접 등록할 수도 있습니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">프리셋 선택</span></td>
                  <td><b>[선택]</b> 버튼을 누르면 VS Code, IntelliJ, Chrome, Excel 등 자주 쓰이는 프로그램 목록이 나타납니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">커스텀 등록</span></td>
                  <td>목록에 없는 경우 <b>[직접 추가]</b> 아이콘을 클릭하여 PC에 설치된 실행 파일(.exe) 중 하나를 직접 선택할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">설정 해제</span></td>
                  <td>등록된 프로그램을 삭제하려면 항목 우측의 <b>[휴지통]</b> 아이콘을 클릭하여 제거할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>

            <div className="guide-info-box">
              <p>💡 <strong>참고:</strong> 프로그램 명칭 뒤에 <code>.exe</code> 확장자가 정확히 붙어있는지 확인해 주십시오. 시스템이 프로그램의 실행 여부를 판단하는 중요한 기준이 됩니다.</p>
            </div>
          </div>
        )
      },
      create_category: { 
        label: '새 작업 추가', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">새로운 작업 유형 만들기</h4>
            <p className="guide-text">기본 제공되는 카테고리 외에 본인만의 작업 유형을 자유롭게 추가하거나 삭제할 수 있습니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">유형 생성</span></td>
                  <td>메뉴 상단의 <b>[+ 새 작업 추가]</b> 버튼을 클릭합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">이름 설정</span></td>
                  <td>나타나는 입력창에 '개인 공부', '영상 편집' 등 자신이 원하는 작업명을 입력하고 확인을 누릅니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">유형 삭제</span></td>
                  <td>불필요해진 작업 유형은 각 작업 유형 카드의 <b>[휴지통]</b> 아이콘을 클릭하여 제거할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>

            <div className="guide-info-box">
              <p>⚠️ <strong>주의:</strong> 작업 유형을 삭제하면 해당 유형에 매칭해두었던 프로그램 설정 정보도 함께 삭제됩니다.</p>
            </div>
          </div>
        )
      },
    }
  },
summary: {
    label: '활동 요약',
    subs: {
      overall: { 
        label: '종합', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">활동 요약: 나의 몰입 데이터 분석</h4>
            <p className="guide-text">
              지난 7일간 수집된 활동 데이터를 단순 요약하여 차트와 보고서 형태로 제공합니다. 자신의 업무 패턴을 객관적으로 파악할 수 있습니다.
            </p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>기능</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">최근 7일간 데이터 분석</span></td>
                  <td>최근 1주일간의 세션 및 이벤트 데이터를 종합하여 객관적인 집중 추이를 그래프와 보고서의 형태로 분석하여 제공합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">데이터 캐싱</span></td>
                  <td>최근 1시간 이내의 데이터를 메모리에 유지하여, 매번 불필요한 로딩을 거치지 않고 즉시 리포트를 확인할 수 있도록 편의성을 제공합니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      graph: { 
        label: '최근 작업 그래프', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">일별 활동 및 집중 강도 그래프</h4>
            <p className="guide-text">시각화된 그래프를 통해 최근 7일간의 요일별 집중 시간과 활동량의 변화를 확인할 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>차트 항목</th>
                  <th>데이터 읽는 법</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">활동량(events)</span></td>
                  <td>키보드/마우스 입력 및 앱 전환 빈도를 측정하여 세션 내 활성 수준을 표시합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">집중 시간</span></td>
                  <td>실제 작업 세션이 유지된 총 시간을 분(minute) 단위로 계산하여 그래프에 반영합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">상세 툴팁</span></td>
                  <td>그래프의 특정 지점에 마우스를 올리면 해당 요일의 정확한 수치를 확인할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      report: { 
        label: '최근 활동 요약 보고서', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">최근 7일간 활동 분석 요약 보고서</h4>
            <p className="guide-text">자신의 활동 내역을 수치화하여 4가지 핵심 지표와 문장으로 요약하여 제공합니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>리포트 항목</th>
                  <th>분석 지표 상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">가장 활발한 요일</span></td>
                  <td>최근 7일 중 <b>세션 내 활동량(이벤트 수)이 가장 높았던 요일</b>을 찾아냅니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">주요 사용 앱</span></td>
                  <td>전체 작업 시간 중 가장 빈번하게 사용되거나 오래 실행된 <b>핵심 프로그램</b>을 식별합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">평균 집중 시간</span></td>
                  <td>7일간의 총 집중 시간을 일평균으로 계산하여 <b>나의 일일 몰입 정도</b>를 보여줍니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">전체 집중 강도</span></td>
                  <td>일평균 집중 시간에 따라 <b>'낮음'</b>부터 <b>'매우 높음'</b>까지 4단계로 몰입 수준을 평가합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">분석 문장</span></td>
                  <td>위 지표들을 조합하여 <b>어떤 요일에 어떤 앱을 중심으로 얼마나 몰입했는지</b> 총평을 요약하여 작성합니다.</td>
                </tr>
              </tbody>
            </table>

            <div className="guide-info-box">
              <p>💡 <strong>참고:</strong> 데이터가 없는 경우 "활동 데이터가 존재하지 않습니다."라는 안내가 표시되며, 새로운 세션을 최소 1개 완료하면 분석이 재개됩니다.</p>
            </div>
          </div>
        )
      },
      layout: { 
        label: '가로/세로로 보기', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">유연한 화면 레이아웃 전환</h4>
            <p className="guide-text">사용자의 모니터 환경이나 취향에 맞춰 화면 구성을 실시간으로 변경할 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>모드</th>
                  <th>화면 구성 특징</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">가로로 보기</span></td>
                  <td>상단에 그래프, 하단에 보고서를 배치하여 <b>한눈에 모든 정보를 비교하기 좋습니다.</b></td>
                </tr>
                <tr>
                  <td><span className="layout-tag">세로로 보기</span></td>
                  <td>좌측에 그래프, 우측에 보고서를 배치하여 <b>모바일이나 세로형 모니터에 최적화됩니다.</b></td>
                </tr>
              </tbody>
            </table>
            <div className="guide-info-box">
              <p>💡 <strong>참고:</strong> 상단 우측의 [보기] 전환 버튼을 클릭하면 설정값이 즉시 반영되며 시스템에 저장되어 이후에도 유지됩니다.</p>
            </div>
          </div>
        )
      },
    }
  },
  feedback: {
    label: '피드백',
    subs: {
      overall: { 
        label: '종합', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">피드백: AI 기반 작업 성과 분석</h4>
            <p className="guide-text">
              종료된 작업 세션의 데이터를 AI가 분석하여 집중도 패턴, 피로도 수치, 그리고 다음 작업을 위한 개선 전략 등을 제안합니다.
            </p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>핵심 기능</th>
                  <th>상세 동작 및 가이드</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">지능형 분석</span></td>
                  <td>세션 중 발생한 앱 사용 패턴과 인지적 임계점 등을 분석하여 <b>Google Gemini 2.0 기반의 맞춤형 AI 피드백 리포트</b>를 생성합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">전역 캐싱</span></td>
                  <td>한 번 분석된 피드백은 앱이 실행되는 동안 메모리에 저장되어, 로그아웃하기 전까지는 다시 조회할 때 <b>대기 시간 없이</b> 즉시 표시됩니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">메뉴 연동</span></td>
                  <td>Overview에서 <b>"최근 작업 피드백"</b> 카드를 클릭하면, 세션 선택 과정을 건너뛰고 <b>가장 최근 세션의 분석 결과</b>로 즉시 연결됩니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">PDF로 피드백 저장</span></td>
                  <td><b>PDF로 저장하기</b> 기능을 통해 현재 피드백 리포트 내용을 PDF 문서로 저장 및 보관할 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      select: { 
        label: '세션 선택', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">분석 대상 세션 선택하기</h4>
            <p className="guide-text">피드백 메뉴 진입 시 나타나는 모달 창에서 분석하고 싶은 과거 기록을 선택할 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>항목</th>
                  <th>동작 및 특징</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">최근 세션</span></td>
                  <td>리스트 최상단에 <b>[최근]</b> 표시와 함께 표시되며, 가장 마지막에 종료된 작업 세션을 의미합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">세션 리스트</span></td>
                  <td><b>최근 20개의 세션 기록</b>이 시간 역순으로 나열됩니다. 날짜와 시간을 확인하여 원하는 세션을 선택하세요.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">다른 세션 선택</span></td>
                  <td>이미 분석 결과를 보고 있더라도 상단의 <b>[↺ 다른 세션 선택]</b> 버튼을 통해 목록으로 돌아갈 수 있습니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
      report: { 
        label: '피드백 결과 확인', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">3가지 관점의 상세 분석 리포트</h4>
            <p className="guide-text">상단 탭을 전환하며 종합적인 성과부터 세부적인 집중도/피로도 지표를 확인할 수 있습니다.</p>
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>분석 탭</th>
                  <th>주요 확인 내용</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">종합</span></td>
                  <td>세션 총평과 함께 <b>주요 성과, 잘한 점, 개선이 필요한 점</b>을 요약된 카드로 보여줍니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">집중도</span></td>
                  <td>최대 연속 몰입 시간, 인지적 임계점, 평균 집중 점수 등 <b>수치화된 집중 지표</b>를 제공합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">피로도</span></td>
                  <td>방해 요소 점유율 그래프와 함께 AI가 제안하는 맞춤형 <b>회복 전략(스트레칭, 수분 섭취 등)</b>을 확인합니다.</td>
                </tr>
              </tbody>
            </table>
            <div className="guide-info-box">
              <p>💡 <strong>참고:</strong> 리포트 결과 확인 이후 <b>[PDF로 저장하기] 버튼</b> 클릭 시 현재 피드백 리포트 내용이 PDF 문서로 자동 정리되어 저장됩니다.</p>
            </div>
          </div>
        )
      },
    }
  },
  userinfo: {
    label: '사용자 정보',
    subs: {
      info: { 
        label: '종합', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">사용자 정보 및 계정 관리</h4>
            <p className="guide-text">현재 로그인된 계정 정보와 관련된 항목을 확인하고 관리할 수 있습니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>항목</th>
                  <th>상세 설명</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">프로필 확인</span></td>
                  <td>좌측 하단의 [사용자 정보] 버튼을 클릭하여 내 <b>이메일</b> 정보와 <b>마지막으로 로그인한 날짜(KST)</b>를 확인할 수 있습니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">로그아웃</span></td>
                  <td>계정 팝업 하단의 <b>로그아웃</b> 버튼을 통해 안전하게 세션을 종료하고 초기화면으로 이동합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">계정 삭제</span></td>
                  <td>계정 팝업 하단의 <b>계정 삭제</b> 버튼을 통해 계정을 <b>영구적으로 삭제</b>할 수 있습니다. </td>
                </tr>
              </tbody>
            </table>

            <div className="guide-info-box">
              <p>⚠️ <strong>주의:</strong> 계정 삭제시 모든 데이터가 영구적으로 삭제되고 다시 복구할 수 없으므로 주의가 필요합니다.</p>
            </div>
          </div>
        )
      },
    }
  },
  theme: {
    label: '다크 모드/라이트 모드',
    subs: {
      toggle: { 
        label: '모드 전환', 
        content: (
          <div className="guide-content">
            <h4 className="guide-section-title">다크 모드/라이트 모드 테마 설정</h4>
            <p className="guide-text">사용자의 작업 환경이나 선호도에 맞게 대시보드의 전체 색상 테마를 다크 모드 또는 라이트 모드로 즉시 변경합니다.</p>
            
            <table className="guide-table">
              <thead>
                <tr>
                  <th style={{ width: '25%' }}>설정 모드</th>
                  <th>특징</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><span className="layout-tag">라이트 모드</span></td>
                  <td><b>밝고 깨끗한 배경</b>으로 낮 시간대나 밝은 환경에서 높은 가독성을 제공합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">다크 모드</span></td>
                  <td><b>어두운 배경</b>을 통해 눈의 피로를 최소화하며, 집중이 필요한 야간 작업에 적합합니다.</td>
                </tr>
                <tr>
                  <td><span className="layout-tag">전환 방법</span></td>
                  <td>좌측 하단의 <b>해/달 모양 아이콘</b>을 클릭하면 실시간으로 테마가 전환됩니다.</td>
                </tr>
              </tbody>
            </table>
          </div>
        )
      },
    }
  }
};

const HelpModal = () => {
  const { isDarkMode, isHelpOpen, closeHelp, activeMenu, scheduleViewMode } = useMainStore(); // activeMenu 추가
  const [activeMainTab, setActiveMainTab] = useState('all');
  const [activeSubTab, setActiveSubTab] = useState('intro');

  useEffect(() => {
    if (isHelpOpen) {
      const menuMapping = {
        'Overview': 'overview',
        '스케줄': 'schedule',
        '작업': 'task',
        '활동 요약': 'summary',
        '피드백': 'feedback'
      };

      const targetTab = menuMapping[activeMenu] || 'all';
      setActiveMainTab(targetTab);

      if (targetTab === 'schedule' && scheduleViewMode) {
        setActiveSubTab(scheduleViewMode);
      } else {
        // 그 외의 경우 해당 메인 카테고리의 첫 번째 서브 탭으로 설정
        setActiveSubTab(Object.keys(helpContent[targetTab].subs)[0]);
      }
    }
  }, [isHelpOpen, activeMenu, scheduleViewMode]); 

  const goToIntro = () => {  // 처음 [소개] 탭으로 즉시 이동하는 함수
    setActiveMainTab('all');
    setActiveSubTab('intro');
  };

 if (!isHelpOpen) return null;

  const handleMainTabChange = (tabKey) => {
    setActiveMainTab(tabKey);
    const firstSubTabKey = Object.keys(helpContent[tabKey].subs)[0];
    setActiveSubTab(firstSubTabKey);
  };

  const currentMainCategory = helpContent[activeMainTab];

  return (
    <div className={`help-modal-overlay ${isDarkMode ? 'dark-theme' : ''}`} onClick={closeHelp}>
      <div className="help-modal-container" onClick={(e) => e.stopPropagation()}>
        <button className="help-modal__close-icon-btn" onClick={closeHelp} aria-label="닫기">
          <svg 
            className="help-modal__close-icon"
            xmlns="http://www.w3.org/2000/svg" 
            viewBox="0 0 24 24" 
            fill="none" 
            stroke="currentColor" 
            strokeWidth="2.5" 
            strokeLinecap="round" 
            strokeLinejoin="round"
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
        
        <aside className="help-modal__sidebar">
          <div className="help-modal__sidebar-header">
            <h2 className="help-modal__title" onClick={goToIntro}>
              <svg 
                className="help-modal__title-icon"
                xmlns="http://www.w3.org/2000/svg" 
                viewBox="0 0 24 24" 
                fill="none" 
                stroke="currentColor" 
                strokeWidth="2.5" 
                strokeLinecap="round" 
                strokeLinejoin="round"
              >
                <path d="M7 8H5a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2l3 3V20h5a2 2 0 0 0 2-2" />
                <path d="M11 4h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-4l-3 3v-3h-2a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" />
              </svg>
              <span>도움말 센터</span>
            </h2>
          </div>
          <nav className="help-modal__nav">
            <ul>
              {Object.entries(helpContent).map(([key, { label }]) => (
                <li
                  key={key}
                  className={`help-modal__nav-item ${activeMainTab === key ? 'active' : ''}`}
                  onClick={() => handleMainTabChange(key)}
                >
                  {label}
                </li>
              ))}
            </ul>
          </nav>
          <div className="help-modal__close-container">
            <button className="help-modal__close-btn" onClick={closeHelp}>도움말 닫기</button>
          </div>
        </aside>
        
        <main className="help-modal__main">
          <header className="help-modal__header">
            <div className="help-modal__sub-tabs">
              {Object.entries(currentMainCategory.subs).map(([key, { label }]) => (
                <button
                  key={key}
                  className={`help-modal__sub-tab-item ${activeSubTab === key ? 'active' : ''}`}
                  onClick={() => setActiveSubTab(key)}
                >
                  {label}
                </button>
              ))}
            </div>
          </header>

          <div className="help-modal__content-area">
            <div className="help-modal__content-inner">
              <h2 className="help-modal__content-main-title">{currentMainCategory.subs[activeSubTab].label}</h2>
              <div className="help-modal__divider"></div>
              <div className="help-modal__text-box">
                {currentMainCategory.subs[activeSubTab].content}
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
};

export default HelpModal;