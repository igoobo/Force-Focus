# 데이터 흐름도

> **작성일**: 2026-03-21
> **최종 업데이트**: 2026-04-19 (ML 파이프라인 다이어그램 정확도 개선)

---

## 1. Frontend ↔ Backend 통신

```mermaid
sequenceDiagram
    participant React as React (Frontend)
    participant Tauri as Tauri IPC
    participant Rust as Rust Backend
    participant DB as SQLite

    Note over React,Rust: 로그인 흐름
    React->>Tauri: open(OAuth URL)
    Note right of Tauri: 시스템 브라우저 → OAuth → Deep Link
    Tauri->>Rust: deep_link 콜백
    Rust->>DB: save_auth_token()
    Rust->>Tauri: emit("login-success", email)
    Tauri->>React: listen("login-success")

    Note over React,Rust: 세션 시작 흐름
    React->>Tauri: invoke("start_session")
    Tauri->>Rust: Lock → 로컬 세션 생성 → Unlock
    Rust->>DB: save_active_session()
    Rust-->>React: Ok(ActiveSessionInfo)
    Rust->>Rust: tokio::spawn(서버 동기화)
```

---

## 2. ML 추론 파이프라인

```mermaid
flowchart LR
    A["1. 활성 창 감지<br/>(vision.rs)"] --> B["2. 시맨틱 토큰화<br/>(vision.rs)"]
    B --> B2["3. global_map.json<br/>context_score 산출"]
    D["4. 입력 통계<br/>(input.rs)"] --> C
    B2 --> C["5. ML 벡터 생성<br/>(app.rs inline)<br/>[f64; 6]"]
    C --> CACHE{"6. Local Cache<br/>확인"}
    CACHE -->|"캐시 히트"| OVERRIDE["vector[0] = 1.0<br/>context 강제 상향"]
    CACHE -->|"캐시 미스"| SCALE
    OVERRIDE --> SCALE["7. Standard Scaling<br/>(mean/scale 정규화)"]
    SCALE --> E["8. ONNX 추론<br/>(inference.rs)"]
    E --> JUDGE{"9. Score 판정"}
    JUDGE -->|">0.0: Inlier"| I_OK["FSM: -2.0<br/>(빠른 회복)"]
    JUDGE -->|">-0.5: Weak"| I_WEAK["FSM: +0.5<br/>(지연 축적)"]
    JUDGE -->|"≤-0.5: Strong"| I_STRONG["FSM: +1.0<br/>(실시간 축적)"]
    I_OK & I_WEAK & I_STRONG --> FSM["10. FSM 상태 전이<br/>(state.rs)"]
    FSM --> J{"개입 판단"}
    J -->|"DoNothing<br/>(Gauge≤0 OR FOCUS)"| K["오버레이 숨김"]
    J -->|"Notification"| L["OS 알림 + 붉은 테두리"]
    J -->|"Overlay"| M["전체 화면 차단"]
```

> **핵심 변경**: Local Cache는 ONNX 추론 **이전**에 동작합니다.
> 캐시 히트 시 추론을 건너뛰는 것이 아니라, `input_vector[0]`을 1.0으로 수정하여 **모델이 Inlier로 판정하도록 유도**합니다.

---

## 3. SQLite 데이터 생명주기

```mermaid
flowchart TB
    subgraph "생성 (Write)"
        W1["cache_event()<br/>5초마다"] --> DB[(SQLite)]
        W2["cache_feedback()<br/>사용자 피드백"] --> DB
        W3["save_auth_token()<br/>로그인"] --> DB
        W4["sync_schedules()<br/>Down-Sync"] --> DB
        W5["sync_tasks()<br/>Down-Sync"] --> DB
    end

    subgraph "읽기 (Read)"
        DB --> R1["get_pending_events()<br/>Up-Sync 대기"]
        DB --> R2["load_auth_token()<br/>인증 확인"]
        DB --> R3["get_active_schedules()<br/>스케줄 체크"]
    end

    subgraph "삭제 (Delete)"
        R1 --> D1["delete_events_by_ids()<br/>Up-Sync 완료 후"]
        D1 --> DB
    end
```

---

## 4. 양방향 서버 동기화

| 방향 | 데이터 | 주기 | 방식 |
|------|--------|------|------|
| **Up-Sync** | Cached Events (50/batch) | 60초 | Lock→Read→Unlock → API POST → Lock→Delete→Unlock |
| **Up-Sync** | Cached Feedbacks (50/batch) | 60초 | 동일 패턴 |
| **Down-Sync** | Tasks, Schedules | 60초 | API GET → Lock→Write→Unlock |

> `sync.rs` (116줄)에서 `api.rs`의 `BackendCommunicator`를 통해 동기화 수행.
