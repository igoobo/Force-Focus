# Force-Focus 기술 문서 인덱스

> **최종 업데이트**: 2026-04-25

---

## Desktop Agent 문서

Desktop Agent(Tauri + Rust + React)의 전체 기술 문서는 아래 경로에서 관리됩니다.

📁 [`desktop-agent/docs/`](../../desktop-agent/docs/)

### 아키텍처 & 백엔드

| # | 문서 | 설명 |
|---|------|------|
| 01 | [아키텍처 개요](../../desktop-agent/docs/01-architecture/overview.md) | 시스템 전체 구조, 핵심 동작 루프, 상태 관리, 모듈 의존 관계 |
| 02 | [Core Layer](../../desktop-agent/docs/02-backend/core.md) | `lib.rs`, `app.rs`, `state.rs`, `input.rs` — FSM + 메인 루프 |
| 02 | [Commands Layer](../../desktop-agent/docs/02-backend/commands.md) | Tauri 커맨드 9파일 — vision, session, auth, task 등 |
| 02 | [AI Layer](../../desktop-agent/docs/02-backend/ai.md) | ONNX 추론 엔진, 모델 업데이트, Local Cache |
| 02 | [Managers Layer](../../desktop-agent/docs/02-backend/managers.md) | SQLite, 스케줄, 동기화, 트레이, 위젯 |
| 02 | [Utils Layer](../../desktop-agent/docs/02-backend/utils.md) | HTTP 클라이언트, 로깅 |

### 프론트엔드 & 설정

| # | 문서 | 설명 |
|---|------|------|
| 03 | [Frontend](../../desktop-agent/docs/03-frontend/frontend.md) | React 18 + TypeScript — 뷰, 컴포넌트, 멀티 윈도우 |
| 04 | [Config](../../desktop-agent/docs/04-config/config.md) | `Cargo.toml`, `tauri.conf.json`, `package.json` 등 |

### 데이터 & API

| # | 문서 | 설명 |
|---|------|------|
| 05 | [데이터 흐름](../../desktop-agent/docs/05-data-flow/data-flow.md) | Frontend↔Backend, ML 파이프라인, SQLite 생명주기, Snapshot 흐름 |
| 06 | [API 레퍼런스](../../desktop-agent/docs/06-api-reference/tauri-commands.md) | Tauri 커맨드 17개 전체 목록 |

### 의사결정 & 로드맵

| # | 문서 | 설명 |
|---|------|------|
| 07 | [의사결정 기록](../../desktop-agent/docs/07-decision-log/decisions.md) | 설계 결정 11건 (보안, 안정성, 기능 등) |
| 08 | [향후 기능](../../desktop-agent/docs/08-future-features/) | Workspace Snapshot 등 |

### 보안

| 문서 | 설명 |
|------|------|
| [보안 정책](../../desktop-agent/docs/security/) | 스케줄 실행 보안, 경로 검증 등 |

---

## ML 파이프라인 문서

ML 모델의 학습, 배포, 추론에 관한 종합 문서입니다.

| 문서 | 설명 |
|------|------|
| [ML 파이프라인](../../desktop-agent/docs/ml_models/ml-pipeline.md) | 훈련(OneClassSVM) → ONNX 변환 → 배포 → 클라이언트 추론 → 피드백 루프 |

---

## 기타 컴포넌트

| 컴포넌트 | 경로 | 설명 |
|----------|------|------|
| Backend API | [`backend/`](../../backend/) | FastAPI 서버 (GCP) |
| Web Dashboard | [`web-dashboard/`](../../web-dashboard/) | 관리 대시보드 |
