# Force-Focus Web dashboard

## 1. 목표와 기능
### 1.1 목표
- **Force-Focus 시스템** : Force-Focus 시스템은 사용자의 작업 환경을 강제로 실행하고, 사용자의 딴짓이 감지될 경우 자동으로 개입하는 기능을 제공하여 사용자의 작업 집중도 향상을 목표로 하는 시스템입니다.

- **통합 관리 대시보드** : 데스크탑 에이전트와 연동되어 사용자의 일정 및 작업 정보, 자신의 작업 세션에 대한 피드백 데이터를 한눈에 조회하고 관리하는 대시보드로서의 역할을 목표로 합니다.

- **실제 데이터 기반의 피드백 제공** : 사용자의 세션 데이터를 활용하여, 사용자의 작업 패턴이나 경향성을 파악하여 이에 맞는 개인화된 형태의 피드백을 제공합니다. (Gemini 2.0 기반 피드백 제공) 

### 1.2 주요 기능
- **일정 및 작업 정보 관리** : 사용자가 자신의 스케줄을 직접 CRUD(생성, 조회, 수정, 삭제) 할 수 있는 인터페이스를 제공합니다.

- **작업 유형별 환경 설정** : 특정 작업 유형에 따라 강제 실행될 프로그램을 사용자가 직접 지정하고 관리할 수 있습니다.

- **Gemini 2.0 기반 활동 피드백** : 사용자의 세션 데이터를 분석하여 AI가 생성한 활동 요약 및 맞춤형 개선 피드백을 제공합니다.

- **실시간 모니터링** : 데스크탑 에이전트에서 사용자의 작업 세션이 완료되면 세션 정보를 실시간으로 반영합니다.

<br>

## 2. 개발 환경 및 배포 URL
### 2.1 개발 환경
- Web Framework
    - Frontend : JavaScript + React
    - Backend : Python + FastAPI

- State Management
    - Zustand (전역 상태 관리 라이브러리)

- Database
    - MongoDB

- Infrastructure & DevOps
    - Containerization : Docker
    - Cloud Platform : Google Cloud Platform (GCP)


### 2.2 배포 URL
- https://34.63.228.213.sslip.io/

### 2.3 시스템 아키텍처

```mermaid
graph LR
    subgraph Client
        A[Web Dashboard - React]
    end
    subgraph Server
        B[API Server - FastAPI]
    end
    subgraph Storage
        C[(Database - MongoDB)]
    end
    subgraph External API
        D[Gemini 2.0 API]
    end

    A <-->|REST API| B
    B <--> C
    B --- D
```

### 2.4 URL 구조
#### 2.4.1 Frontend URL (SPA)
본 프로젝트는 **Single Page Application (SPA)** 방식을 채택하여 대시보드 내 모든 페이지 전환 및 기능 수행이 루트 경로 내에서 동적으로 이루어집니다.

| App | URL | Note |
|:---|:---|:---|
| Web Dashboard | `/` | Zustand 전역 상태 관리에 의한 컴포넌트 스위칭 렌더링 |

<br>

#### 2.4.2 Backend API Endpoints (FastAPI)
백엔드와의 통신은 `axios` 인스턴스를 통해 수행되며, 모든 요청은 `/api/v1`을 Base URL로 사용합니다.

**1) Schedule API**
| Method | Endpoint | 설명 | 로그인 권한 |
|:---:|:---|:---|:---:|
| `GET` | `/schedules/` | 전체 일정 목록 조회 | ✅ |
| `POST` | `/schedules/` | 새로운 일정 생성 | ✅ |
| `GET` | `/schedules/{id}` | 특정 일정 정보 조회 | ✅ |
| `PUT` | `/schedules/{id}` | 특정 일정 정보 수정 | ✅ |
| `DELETE` | `/schedules/{id}` | 특정 일정 삭제 | ✅ |

<br>

**2) Task API**
| Method | Endpoint | 설명 | 로그인 권한 |
|:---:|:---|:---|:---:|
| `GET` | `/tasks/` | 정의된 작업 유형 목록 조회 | ✅ |
| `POST` | `/tasks/` | 새로운 작업 유형 생성 | ✅ |
| `GET` | `/tasks/{id}` | 특정 작업 유형 상세 정보 조회 | ✅ |
| `PUT` | `/tasks/{id}` | 작업 유형 설정 업데이트 | ✅ |
| `DELETE` | `/tasks/{id}` | 작업 유형 삭제 | ✅ |

<br>

**3) Session & Activity API**
| Method | Endpoint | 설명 | 로그인 권한 |
|:---:|:---|:---|:---:|
| `GET` | `/sessions/` | 사용자 활동 세션 기록 조회 | ✅ |
| `GET` | `/events/` | 세션 내 발생한 상세 이벤트 로그 조회 | ✅ |

<br>

**4) Feedback API**
| Method | Endpoint | 설명 | 로그인 권한 |
|:---:|:---|:---|:---:|
| `GET` | `/insight` | Gemini 2.0 기반 피드백 제공 | ✅ |

<br>

**5) Authentication API**
| Method | Endpoint | 설명 | 로그인 권한 |
|:---:|:---|:---|:---:|
| `POST` | `/auth/login` | 사용자 인증 및 토큰 발급 | - |
| `POST` | `/auth/logout` | 세션 종료 및 로그아웃 | ✅ |

<br>

## 3. 프로젝트 구조 및 설계
### 3.1 프로젝트 폴더 구조
웹 대시보드의 프론트엔드 소스코드는 기능별 모듈화를 위해 다음과 같은 구조로 설계되었습니다.

```text
📦src
 ┣ 📂api               # Backend 연동을 위한 Axios 인스턴스 및 API 정의
 ┃ ┣ 📜authApi.js
 ┃ ┣ 📜axiosInstance.js
 ┃ ┣ 📜scheduleApi.js
 ┃ ┣ 📜sessionApi.js
 ┃ ┗ 📜taskApi.js
 ┣ 📂assets
 ┃ ┗ 📜react.svg
 ┣ 📂components        # 재사용 가능한 UI 컴포넌트
 ┃ ┣ 📂layout          # 공통 레이아웃 컴포넌트
 ┃ ┃ ┣ 📂Help          # 도움말 섹션
 ┃ ┃ ┣ 📂InfoBox       # 정보 표시 박스
 ┃ ┃ ┣ 📂MenuBar       # 사이드/네비게이션 바
 ┃ ┃ ┗ 📂TitleBar      # 상단 타이틀 바
 ┃ ┣ 📂login           # 로그인 페이지
 ┃ ┗ 📂menu            # 메인 기능별 상세 컴포넌트
 ┃   ┣ 📂ActivitySummary  # 활동 요약 메뉴
 ┃   ┣ 📂Feedback         # 피드백 메뉴
 ┃   ┣ 📂Overview         # 대시보드 개요
 ┃   ┣ 📂Schedule         # 일정 관리 메뉴
 ┃   ┗ 📂Task             # 작업 유형 설정 메뉴
 ┣ 📂hooks             # 커스텀 훅
 ┃ ┣ 📜useSchedules.js
 ┃ ┗ 📜useTasks.js
 ┣ 📜App.css           
 ┣ 📜App.jsx           
 ┣ 📜index.css         
 ┣ 📜main.jsx          
 ┗ 📜MainStore.jsx     # Zustand를 이용한 전역 상태(Menu, Auth 등) 관리
```

<br>

## 4. 데이터베이스 설계 (ERD)
본 프로젝트는 NoSQL 데이터베이스인 **MongoDB**를 사용하여 비정형 활동 데이터와 유연한 일정 정보를 관리합니다. 각 컬렉션의 구조와 관계는 다음과 같습니다.

### 4.1 Entity Relationship Diagram

```mermaid
erDiagram
    USER ||--o{ SCHEDULE : manages
    USER ||--o{ TASK : defines
    USER ||--o{ SESSION : performs
    SESSION ||--o{ EVENT : contains
    SESSION ||--|| FEEDBACK : generates

    SCHEDULE {
        string id PK
        string name
        string task_id FK
        string description
        string start_date
        string end_date
        string start_time
        string end_time
        int[] days_of_week
        boolean is_active
    }

    TASK {
        string id PK
        string name
        string description
        string status
        string target_executable
    }

    SESSION {
        string id PK "(_id)"
        string client_session_id FK
        string start_time
        float duration
        int event_count
    }

    EVENT {
        string id PK "(_id)"
        string session_id FK
        string app_name
        date timestamp
    }
``` 

### 4.2 컬렉션별 상세 명세
**1) Schedules (일정 관리)**
사용자가 웹 대시보드에서 설정한 작업 계획 데이터입니다.
* `id`: 일정 고유 식별자 (String / MongoDB ObjectId)
* `name`: 일정 제목 (String)
* `task_id`: 연결된 작업 유형의 ID (String)
* `description`: 일정에 대한 추가 설명 (String)
* `start_date / end_date`: 시작 및 종료 날짜 (String, "YYYY-MM-DD" 포맷)
* `start_time / end_time`: 시작 및 종료 시간 (String, "HH:mm:ss" 포맷)
* `days_of_week`: 반복 요일 (Array<Int>, 0:일 ~ 6:토)
* `is_active`: 활성화 여부 (Boolean)

<br>

**2) Tasks (작업 설정)**
작업 유형별 프로필 및 프로그램 제어 환경을 정의합니다.
* `id`: 작업 고유 식별자 (String)
* `name`: 작업 유형 명칭 (String)
* `description`: 작업 환경 설명 (String)
* `status`: 활성화 여부 (String)
* `target_executable`: 강제 실행 프로그램 목록 (String, 콤마(,)로 구분)

<br>

**3) Sessions (활동 세션)**
데스크탑 에이전트로부터 수집되어 저장된 실제 작업 이력 데이터입니다.
* `_id`: 세션 고유 식별자 (String / MongoDB ObjectId)
* `client_session_id`: 데스크탑 에이전트에서 생성하는 session_id 형식, 각 세션별 events 정보 구분자 (String)
* `start_time`: 각 세션이 시작되는 시간 (String, YYYY-MM-DDTHH:mm:ssZ" 형식)
* `duration`: 각 세션의 지속 시간 (Float)
* `event_count` : 각 세션에서 발생한 입력 이벤트 횟수 (Int)

<br>

**4) Events (상세 활동 로그)**
세션 내에서 발생한 구체적인 입력 이벤트와 관련된 로그입니다.
* `_id`: 이벤트 고유 식별자 (String / UUID 형식)
* `session_id`: Client Session ID와의 매핑 식별자 (String)
* `app_name`: 실행 프로세스 명 (예: `chrome.exe`)
* `timestamp`: 이벤트 발생 시각 (Date / $date 포맷)

<br>

## 5. 메인 기능 상세
- 본 웹 대시보드에서는 **MainStore**에서 Zustand를 통하여 관리되는 상태에 따라 각기 다른 기능을 수행하는 컴포넌트를 렌더링합니다. 
- 각 메뉴의 핵심 기능은 다음과 같습니다.

### 5.1 Overview (대시보드 개요)
* **기능**: 사용자의 현재 상태와 오늘의 요약 정보를 한눈에 제공합니다.
* **상세**: 
    * 현재 활성화된 일정 및 진행 중인 작업 정보 표시
    * 데스크탑 에이전트로부터 수집된 최근 활동 로그의 간략한 대시보드 출력
```mermaid
graph LR
    A[사용자] --> B{Overview 확인}
    B --> C[최근 7일간의 작업 요약 표시]
    B --> D[최근 활동 요약 그래프]
    B --> E[오늘 기준 스케줄 확인]
```

<br>

### 5.2 Schedule (일정 관리)
* **기능**: 사용자의 작업 계획을 설정하고 데이터베이스에 저장합니다.
* **상세**: `useSchedules` 커스텀 훅을 통해 `scheduleApi`와 통신하며 CRUD 기능을 수행합니다.
    * **조회**: 등록된 모든 일정을 리스트 형태로 출력
    * **생성/수정**: 작업명, 시작/종료 시간, 반복 설정 등을 입력 및 백엔드 동기화
    * **삭제**: 불필요한 일정 제거 및 실시간 UI 업데이트
```mermaid
graph TD
    A[사용자] --> B[일정 입력/수정]
    B --> C[DB 저장 및 에이전트 동기화]
    C --> D[목록에 실시간 반영]
```

<br>

### 5.3 Task (작업 설정)
* **기능**: 특정 작업 유형에 따른 시스템 강제 사항(프로그램 실행 등)을 정의합니다.
* **상세**: `useTasks` 훅을 사용하여 사용자가 정의한 작업 환경 설정을 관리합니다.
    * **강제 실행 설정**: 특정 Task 시작 시 자동으로 실행될 프로그램 경로 및 환경 변수 지정
    * **환경 제어**: 해당 작업 중 허용/차단할 애플리케이션 리스트 구성
```mermaid
graph LR
    A[사용자] --> B[작업 유형 추가 및 선택]
    B --> C[강제 실행 프로그램 등록]
    C --> D[작업 유형별 정보 DB 저장]
    D --> E[에이전트 실행 정책 반영]
```

<br>

### 5.4 Activity Summary (활동 요약)
* **기능**: 데스크탑 에이전트가 수집한 세션 및 이벤트 데이터를 시각화합니다.
* **상세**: `sessionApi`를 호출하여 특정 기간의 활동 데이터를 호출합니다.
    * `getSessions`: 사용자의 집중 세션 시작/종료 시간 및 지속 시간 데이터 활용
    * `getEvents`: 세션 내에서 발생한 구체적인 앱 전환, 웹사이트 방문 기록 등 이벤트 로그 출력
```mermaid
graph TD
    A[에이전트 데이터 수집] --> B[DB 저장]
    B --> C{웹 대시보드 조회}
    C --> D[집중 시간 통계 시각화]
    C --> E[활동 요약 보고서 제공]
```

<br>

### 5.5 Feedback (피드백)
* **기능**: **Gemini 2.0** 모델을 활용하여 사용자의 활동을 분석하고 맞춤형 개선안을 제시합니다.
* **상세**:
    * 수집된 활동 데이터(Activity Summary)를 기반으로 AI 분석 요청
    * 사용자의 집중도 패턴 분석, 딴짓 빈도, 작업 효율성에 대한 정성적/정량적 피드백 생성
    * 향후 집중력 향상을 위한 구체적인 작업 환경 개선 가이드 제공
```mermaid
graph LR
    A[활동 데이터] --> B[Gemini 2.0 AI 기반 분석]
    B --> C{피드백 생성}
    C --> D[종합 피드백 제공]
    C --> E[집중도 피드백 제공]
    C --> F[피로도 피드백 제공]
```

<br>

### 5.6 Authentication (로그인 및 인증)
* **기능**: 보안을 위한 사용자 인증 및 세션 관리를 수행합니다.
* **구현**:
    * `authApi`를 통한 로그인 처리 및 JWT(JSON Web Token) 발급
    * `axiosInstance`의 인터셉터를 활용하여 모든 요청 헤더에 토큰 자동 포함
    * 401 에러 발생 시(세션 만료) 자동으로 상태 초기화 및 로그인 화면으로 리다이렉트
```mermaid
graph TD
    A[사용자 로그인] --> B[JWT 토큰 발급]
    B --> C[브라우저 저장]
    C --> D{모든 API 요청}
    D -->|토큰 유효| E[정상 서비스 이용]
    D -->|토큰 만료| F[자동 로그아웃/재로그인]
```

<br>

## 6. 메인 기능별 상세 아키텍처
### 6.1 Schedule (일정 관리)
- 스케줄 진입 시 서버로부터 전체 일정을 불러와 화면에 렌더링합니다. 사용자는 직관적인 모달 인터페이스를 통해 일정의 추가, 수정, 삭제를 수행하며, 각 작업 성공 시 프론트엔드 상태를 서버 데이터와 즉각적으로 동기화하여 최신 상태를 유지합니다.
```mermaid
graph TD
    %% 1. 초기 로드 (Read) 및 에러 처리
    Start([사용자: 스케줄 메뉴 진입]) --> Fetch[fetchSchedules 호출]
    Fetch --> API_Get[GET /api/v1/schedules]
    API_Get -- "성공" --> Norm_R[데이터 정규화: normalizeSchedule]
    API_Get -- "실패 (4xx/500)" --> Err_Get[에러 상태 저장 및 콘솔 출력]
    Norm_R --> UI_List[화면 렌더링: 일/주/월/목록 뷰]
    Err_Get --> UI_Err[화면: 에러 메시지 표시]

    %% 2. 일정 추가 (Create) 및 에러 처리
    UI_List --> Add_Click{사용자: + 일정 추가 클릭}
    Add_Click --> Add_Modal[ScheduleAddModal 오픈]
    Add_Modal --> Add_Save[addSchedule 호출]
    Add_Save --> API_Post[POST /api/v1/schedules/]
    API_Post -- "성공" --> Sync_A[fetchSchedules로 목록 동기화]
    API_Post -- "실패" --> Alert_A[실패 원인 분석 및 alert 노출]
    Alert_A --> Add_Modal
    Sync_A --> UI_List

    %% 3. 일정 수정 (Update) 및 에러 처리
    UI_List --> Edit_Click{사용자: 일정 항목 클릭 - 일정 수정}
    Edit_Click --> Edit_Modal[ScheduleEditModal 오픈]
    Edit_Modal --> Edit_Save[updateSchedule 호출]
    Edit_Save --> API_Put[PUT /api/v1/schedules/id]
    API_Put -- "성공" --> Sync_U[fetchSchedules로 목록 동기화]
    API_Put -- "실패" --> Alert_U[수정 실패 alert 노출]
    Alert_U --> Edit_Modal
    Sync_U --> UI_List

    %% 4. 일정 삭제 (Delete) 및 에러 처리
    UI_List --> Del_Click{사용자: - 일정 삭제 클릭}
    Del_Click --> Del_Modal[ScheduleDeleteModal 오픈]
    Del_Modal --> Del_Confirm[deleteSchedule 호출]
    Del_Confirm --> API_Del[DELETE /api/v1/schedules/id]
    API_Del -- "성공" --> Sync_D[fetchSchedules로 목록 동기화]
    API_Del -- "실패" --> Alert_D[삭제 실패 alert 노출]
    Alert_D --> UI_List
    Sync_D --> UI_List

    %% 스타일 정의
    style Start fill:#f9f,stroke:#333,stroke-width:2px
    style UI_List fill:#bbf,stroke:#333,stroke-width:2px
    style UI_Err fill:#f66,stroke:#333
    style Err_Get fill:#ff9999,stroke:#333
    style Alert_A fill:#ff9999,stroke:#333
    style Alert_U fill:#ff9999,stroke:#333
    style Alert_D fill:#ff9999,stroke:#333
```
<br>

### 6.2 Task (작업 설정)
- 작업 설정 메뉴는 초기 로드 시 저장된 데이터가 없으면 기본 작업 목록을 자동으로 생성하여 초기화를 지원합니다. 사용자가 그리드 상에서 편집한 내용은 즉시 서버에 반영되지 않고 로컬 상태(isDirty)로 관리되며, '저장하기' 버튼을 클릭할 때 유효성 검사를 거쳐 일괄적으로 백엔드에 업데이트되는 방식으로 동작합니다.
```mermaid
graph TD
    %% 1. 초기 로드 및 자동 초기화 (Read)
    Start([사용자: 작업 설정 메뉴 진입]) --> Fetch[fetchTasks 호출]
    Fetch --> API_Get[GET /api/v1/tasks]
    
    API_Get -- "성공 (데이터 없음)" --> Init[initializeDefaultTasks 실행]
    Init --> API_Init[기본 5개 작업 POST 요청]
    API_Init -- "성공" --> Sync_I[fetchTasks 재호출]
    API_Init -- "실패" --> Err_Init[콘솔 에러 출력 및 초기화 중단]

    API_Get -- "성공 (데이터 존재)" --> Norm[normalizeTask: UI 데이터 변환]
    API_Get -- "실패 (네트워크/서버)" --> Err_Fetch[로딩 종료 및 기존 상태 유지]
    
    Sync_I --> Norm
    Norm --> UI_List[화면 렌더링: 작업 그리드]

    %% 2. 로컬 편집 (Local Editing)
    UI_List --> Edit{사용자 편집 행위}
    Edit --> Add_L[새 작업 추가/수정/삭제]
    Add_L --> Mark_D[isDirty: true 설정]
    Mark_D --> UI_List

    %% 3. 일괄 저장 프로세스 (Batch Save)
    Mark_D --> Save_Req([저장하기 버튼 클릭])
    Save_Req --> Valid{유효성 검사: 빈 칸 확인}
    
    Valid -- "미입력 존재" --> Alert_V[경고창: 모든 빈 칸 완성 요청]
    Alert_V --> UI_List
    
    Valid -- "통과" --> Batch[handleSave: Promise.all 실행]
    
    subgraph "Batch API Operation"
        Batch --> API_Del[DELETE 요청]
        Batch --> API_Add[POST 요청]
        Batch --> API_Upd[PUT 요청]
    end

    API_Del & API_Add & API_Upd -- "모두 성공" --> Save_Ok[성공 알림 및 isDirty 해제]
    API_Del & API_Add & API_Upd -- "하나라도 실패" --> Save_Fail[에러 알림: 데이터 저장 중 오류 발생]

    Save_Ok --> Sync_F[fetchTasks: 최종 동기화]
    Save_Fail --> UI_List[편집 상태 유지: 재시도 가능]
    Sync_F --> UI_List

    %% 스타일 정의
    style Start fill:#f9f,stroke:#333,stroke-width:2px
    style UI_List fill:#bbf,stroke:#333,stroke-width:2px
    style Err_Init fill:#ff9999,stroke:#333
    style Err_Fetch fill:#ff9999,stroke:#333
    style Alert_V fill:#ff9999,stroke:#333
    style Save_Fail fill:#f66,stroke:#333,stroke-width:2px
    style Save_Ok fill:#dfd,stroke:#333
```
<br>

### 6.3 Activity Summary (활동 요약)
- 인증된 사용자의 최근 7일간 세션과 이벤트 데이터를 병렬로 호출하여 분석 효율성을 극대화합니다. 수집된 원천 데이터를 바탕으로 요일별 활동량, 최다 사용 앱, 집중 강도 등을 프론트엔드 로직으로 산출하며, 이를 AreaChart와 요약 보고서 형태로 시각화하여 제공합니다.
```mermaid
graph TD
    %% 1. 초기 진입 및 인증 확인
    Start([사용자: 활동 요약 메뉴 진입]) --> Check_Auth{인증 토큰 확인}
    Check_Auth -- "토큰 없음" --> Warn[경고 출력 및 중단]
    Check_Auth -- "토큰 있음" --> Set_Loading[loading: true 설정]

%% 2. 데이터 수집 단계 (API)
Set_Loading --> Range[최근 7일 날짜 범위 계산]
Range --> API_Parallel[Promise.all 병렬 호출]

subgraph "데이터 호출 (API)"
    API_Parallel --> Get_Sessions[GET /api/v1/sessions?limit=200]
    API_Parallel --> Get_Events[GET /api/v1/events?start_date&end_date]
end

%% 3. 비정상 흐름 (Error Handling)
Get_Sessions & Get_Events -- "API 호출 실패" --> Err_Catch[에러 로그 출력 및 로딩 종료]
Err_Catch --> UI_Fallback[기본 0 데이터 차트 표시]

%% 4. 데이터 분석 로직 (Analysis)
Get_Sessions & Get_Events -- "성공" --> Init_Map[7일치 analysisMap 초기화]

subgraph "프론트엔드 분석 로직 (ActivityStore)"
    Init_Map --> Loop_S[세션 순회: 요일별 지속시간/활동량 합산]
    Loop_S --> Loop_E[이벤트 순회: 앱 이름별 빈도수 계산]
    Loop_E --> Calc_Metrics[지표 산출: 최다 사용 앱/가장 바쁜 요일/평균 시간]
    Calc_Metrics --> Calc_Level[집중 강도 판별: 4단계 - 낮음/보통/높음/매우 높음]
end

%% 5. UI 업데이트 및 시각화
Calc_Level --> Set_Stats[stats 상태 업데이트 및 loading: false]
Set_Stats --> UI_Render[ActivitySummary 화면 렌더링]

UI_Render --> Chart[ActivityChart: AreaChart 시각화]
UI_Render --> Report[ReportItem: 분석 요약 보고서 출력]

%% 스타일 정의
style Start fill:#f9f,stroke:#333,stroke-width:2px
style UI_Render fill:#bbf,stroke:#333,stroke-width:2px
style Err_Catch fill:#ff9999,stroke:#333
style API_Parallel fill:#dfd,stroke:#333
```
<br>

### 6.4 Feedback (피드백)
- 수신된 분석 데이터를 바탕으로 사용자로부터 선택된 세션에 대하여 종합, 집중도, 피로도라는 세 가지 관점으로 피드백을 제공합니다. 특히 피로도 분석에서는 방해 요소에 대응하는 회복 전략을 자동으로 매핑하며, 제안된 전략이 부족할 경우 시스템이 자체적으로 휴식 권고 사항을 보완하고 애니메이션을 통해 시각적 이해도를 높입니다.
```mermaid
graph TD
    Start([사용자: 피드백 메뉴 진입]) --> Fetch_List[전체 세션 목록 로드]
    Fetch_List --> UI_Select[화면: 세션 선택 리스트 출력]
    UI_Select --> User_Click{사용자: 특정 세션 선택}

    User_Click --> Check_Cache{선택한 세션 분석 정보가<br/>로컬 캐시에 있는가?}
    Check_Cache -- "NO" --> API_Insight[GET /api/v1/insight/session-id 호출]
    API_Insight --> AI_Process[백엔드: AI 분석 수행 및 반환]
    AI_Process --> Save_Cache[분석 데이터 로컬 캐싱]
    
    Check_Cache -- "YES" --> Data_In[데이터 수신 완료]
    Save_Cache --> Data_In

    Data_In[데이터 수신 완료] --> Tab_Check{현재 탭 모드?}
    Tab_Check -- "종합" --> Summary[3가지 분석 카드 렌더링: 요약/긍정/개선]

Tab_Check -- "집중도" --> Focus[집중 지표 출력: 최대 몰입/임계점/평균 점수]

Tab_Check -- "피로도" --> Fatigue[방해 요소 분석 및 회복 전략]
Fatigue --> Fill_Logic{전략 데이터가 2개 이하인가?}
Fill_Logic -- "YES" --> Default_Add[기본 '시각적 휴식/스트레칭' 전략 강제 보완]
Fill_Logic -- "NO" --> Icon_Map[getStrategyIcon: 아이콘 매핑]

Icon_Map --> Gauge_Anim[방해 점유율 게이지 너비 애니메이션 적용]

Summary & Focus & Gauge_Anim --> UI_Done[사용자에게 최종 정보 제공]
```
<br>

### 6.5 Authentication (로그인 및 인증)
- Google OAuth를 통한 간편 로그인 방식을 채택하여 사용자의 신원을 안전하게 검증합니다. 백엔드에서 ID 토큰의 유효성을 확인한 후 MongoDB에 사용자 정보를 저장하거나 업데이트하며, 최종적으로 발급된 JWT 토큰을 로컬 스토리지에 저장하여 이후 모든 API 요청의 인증 수단으로 활용합니다.
```mermaid
graph TD
    %% 1. 프론트엔드 로그인 시작
    Start([사용자: 로그인 페이지 접속]) --> Google_Btn[Google로 로그인하기 버튼 클릭]
    Google_Btn --> Google_OAuth{Google OAuth 인증}

Google_OAuth -- "인증 실패" --> Alert_Err[구글 로그인 실패 알림]
Google_OAuth -- "인증 성공 (Credential 반환)" --> FE_Verify[handleGoogleSuccess 실행]

%% 2. 백엔드 검증 요청
FE_Verify --> API_Verify[POST /api/v1/auth/google/verify]
API_Verify --> BE_Check{백엔드: ID Token 검증}

%% 3. 백엔드 로직 (auth.py 기반)
BE_Check -- "유효하지 않은 토큰" --> BE_401[HTTPException 401 반환]
BE_Check -- "유효한 토큰" --> DB_Search{DB: 기존 사용자 확인}

subgraph "백엔드 사용자 처리 (MongoDB)"
    DB_Search -- "이미 가입된 사용자" --> Update_Login[last_login_at 업데이트]
    DB_Search -- "신규 사용자" --> Create_User[새 User 객체 생성 및 저장]
    Update_Login & Create_User --> Issue_Token[JWT Access/Refresh Token 생성]
end

%% 4. 프론트엔드 사후 처리
Issue_Token --> FE_Success[프론트엔드: 응답 수신]
BE_401 --> FE_Fail[에러 알림 및 로딩 중단]

FE_Success --> Storage[localStorage에 토큰 저장]
Storage --> Redirect[onLoginSuccess 호출: 메인 대시보드 진입]

%% 스타일 정의
style Start fill:#f9f,stroke:#333,stroke-width:2px
style Google_OAuth fill:#fff,stroke:#4285F4,stroke-width:2px
style DB_Search fill:#dfd,stroke:#333
style FE_Fail fill:#ff9999,stroke:#333
style BE_401 fill:#ff9999,stroke:#333
```

## 7. 요약 및 기대 효과
- Force-Focus 프로젝트는 사용자가 집중할 수밖에 없는 작업 환경을 강제로 조성하여 생산성을 극대화하는 솔루션으로, 기존의 단순 시간 측정(Time-tracker) 앱이나 방해 금지 모드 등과 달리, **실행 강제**라는 심화적인 접근 방식과 **AI 기반 정성 피드백 분석**이라는 기능을 결합하였다는 점에서 의의가 있습니다. 웹 대시보드는 이 프로젝트의 중앙 관제소 역할을 수행함으로써, 데스크탑 에이전트와 연동되어 일정/작업 정보 관리 및 시각화, AI 기반 피드백을 제공하는 역할을 수행합니다.
- 사용자는 이 웹 대시보드의 산출물을 통해서 자신의 집중 임계점을 객관적으로 확인하고, AI가 제안하는 회복 전략을 통해 자신에게 맞는 집중 환경을 조성할 수 있으며, 장기적으로는 더 나은 작업 패턴을 형성할 수 있을 것이 기대됩니다.