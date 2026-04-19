# Tauri 커맨드 API 레퍼런스

> **작성일**: 2026-03-21
> **최종 업데이트**: 2026-04-19 (Phase 3 모듈 분리 반영, 등록 커맨드 16개 완전 대조)

---

## 컨텍스트

모든 Tauri 커맨드는 Frontend에서 `invoke('command_name', { args })` 형태로 호출됩니다.
Backend에서는 `#[command]` 매크로로 정의되며, `lib.rs`의 `invoke_handler`에 등록됩니다.

> 현재 **16개** 커맨드가 등록되어 있습니다 (`lib.rs:193-210`).

---

## 1. 인증 (`commands/auth.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `login` | `access_token: String`, `refresh_token: String`, `user_email: String`, `user_id: String` | `Result<(), String>` | LSN에 토큰 저장 (XOR 난독화) |
| `logout` | — | `Result<(), String>` | LSN에서 토큰 삭제 |
| `check_auth_status` | — | `Result<Option<String>, String>` | 저장된 토큰 확인 → 이메일 반환 (자동 로그인) |

---

## 2. 세션 관리 (`commands/session.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `start_session` | `task_id: Option<String>`, `goal_duration: u32` | `Result<ActiveSessionInfo, String>` | 로컬 세션 생성 + spawn(서버 동기화) |
| `end_session` | `user_evaluation_score: u8` | `Result<(), String>` | 세션 종료 + FSM 리셋 + 오버레이 숨김 |
| `get_current_session_info` | — | `Result<Option<ActiveSessionInfo>, String>` | 현재 세션 상태 조회 (위젯 동기화용) |

---

## 3. 피드백 (`commands/session.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `submit_feedback` | `feedback_type: String` | `Result<(), String>` | FSM 즉시 리셋 + LSN 캐시 + spawn(서버 전송) |

> `feedback_type` 값: `"is_work"` (오탐지 신고 → Local Cache 등록), `"distraction_ignored"` (업무 복귀)

---

## 4. 태스크 (`commands/task.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `get_tasks` | — | `Result<Vec<Task>, String>` | LSN에서 태스크 목록 조회 (Down-Sync 데이터) |

---

## 5. 시스템 정보 (`commands/system.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `get_all_processes_summary` | — | `Result<Vec<ProcessSummary>, String>` | 전체 프로세스 목록 |

---

## 6. 윈도우 제어 (`commands/window.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `show_overlay` | — | `Result<(), String>` | 오버레이 윈도우 표시 + always_on_top |
| `hide_overlay` | — | `Result<(), String>` | 오버레이 숨김 + FSM 수동 리셋 (manual_reset) |
| `set_overlay_ignore_cursor_events` | `ignore: bool` | `Result<(), String>` | 마우스 클릭 통과/차단 전환 (notification↔blocking) |

---

## 7. 입력 통계 (`commands/input.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `get_input_frequency_stats` | — | `Result<InputStats, String>` | 입력 통계 조회 (deep copy) |

---

## 8. 비전 센서 (`commands/vision.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `get_current_active_window_info` | — | `Result<WindowInfo, String>` | 활성 창 정보 (제목, 프로세스명, 경로) |
| `get_visible_windows` | — | `Result<Vec<WindowInfo>, String>` | 현재 보이는 모든 창 목록 |

---

## 9. ML 모델 (`commands/ml.rs`)

| 커맨드 | 파라미터 | 반환값 | 설명 |
|--------|---------|--------|------|
| `check_model_update` | `token: String` | `Result<bool, String>` | 서버에서 최신 모델 버전 확인 + 다운로드 |

---

## 등록 커맨드 전체 요약 (16개)

| # | 커맨드 | 모듈 | 동기/비동기 |
|---|--------|------|------------|
| 1 | `login` | `auth.rs` | sync |
| 2 | `logout` | `auth.rs` | sync |
| 3 | `check_auth_status` | `auth.rs` | sync |
| 4 | `start_session` | `session.rs` | async |
| 5 | `end_session` | `session.rs` | async |
| 6 | `get_current_session_info` | `session.rs` | sync |
| 7 | `submit_feedback` | `session.rs` | async |
| 8 | `get_tasks` | `task.rs` | sync |
| 9 | `get_all_processes_summary` | `system.rs` | sync |
| 10 | `show_overlay` | `window.rs` | sync |
| 11 | `hide_overlay` | `window.rs` | sync |
| 12 | `set_overlay_ignore_cursor_events` | `window.rs` | sync |
| 13 | `get_input_frequency_stats` | `input.rs` | sync |
| 14 | `get_current_active_window_info` | `vision.rs` | sync |
| 15 | `get_visible_windows` | `vision.rs` | sync |
| 16 | `check_model_update` | `ml.rs` | async |

> **참고**: `get_semantic_tokens` (vision.rs)는 `#[command]`로 정의되어 있지만 `invoke_handler`에 **미등록**입니다. Core Loop 내부에서 직접 호출되는 헬퍼 함수로 사용됩니다.
