# Config & Infrastructure — 코드 리뷰 & 기술 문서

> **범위**: `Cargo.toml`, `tauri.conf.json`, `vite.config.ts`, `tsconfig.json`, `package.json`, `tests/integration_test.rs`
> **리뷰 일자**: 2026-03-21
> **최종 업데이트**: 2026-04-19 (Phase 3/4 변경 반영, CSP/identifier 수정 확인)

---

## 1. 파일별 상세 리뷰

---

### 1.1 `Cargo.toml` (106줄) — Rust 의존성

#### 의존성 목록 (25개)

| # | 크레이트 | 버전 | 용도 | 비고 |
|---|---------|------|------|------|
| 1 | `tauri` | 2 | 앱 프레임워크 | `tray-icon` feature |
| 2 | `serde`/`serde_json` | 1 | 직렬화 | ✅ |
| 3 | `sysinfo` | 0.37.2 | 시스템 정보 | ✅ |
| 4 | `rdev` | 0.5.3 | 입력 감지 | ✅ |
| 5 | `screenshots` | 0.8.10 | 스크린샷 | ⚠️ 코드에서 미사용 |
| 6 | `active-win-pos-rs` | 0.9.1 | 활성 창 | ✅ |
| 7 | `chrono` | 0.4.42 | 시간 | ✅ |
| 8 | `reqwest` | 0.12 | HTTP | stream, json, multipart |
| 9 | `tauri-plugin-notification` | 2 | 알림 | ✅ |
| 10 | `rusqlite` | 0.37.0 | SQLite | bundled |
| 11 | `lazy_static` | 1.5.0 | 정적 변수 | 테스트용 |
| 12 | `tokio` | 1.48.0 | 비동기 | full features |
| 13 | `futures-util` | 0.3.31 | 스트림 | ✅ |
| 14 | `anyhow` | 1.0 | 에러 처리 | ✅ |
| 15 | `dotenv` | 0.15.0 | 환경변수 | ✅ |
| 16 | `uuid` | 1.18.1 | UUID v4 | ✅ |
| 17 | `windows` | 0.58 | Win32 API | 5 features |
| 18 | `regex` | 1.12.2 | 시맨틱 태깅 | ✅ |
| 19 | `ort` | 2.0.0-rc.11 | ONNX Runtime | ⚠️ RC 버전 |
| 20 | `ndarray` | 0.17.2 | 행렬 연산 | ONNX 텐서 생성용 |
| 21 | `tauri-plugin-opener` | 2 | 시스템 브라우저 열기 | OAuth 로그인 |
| 22 | `tauri-plugin-deep-link` | 2 | Deep Link 수신 | OAuth 콜백 |
| 23 | `tauri-plugin-shell` | 2 | 셸 커맨드 | ✅ |
| 24 | `tauri-plugin-autostart` | 2 | OS 자동시작 | 비모바일 전용 |
| 25 | `tauri-plugin-single-instance` | 2 | 단일 인스턴스 | 비모바일 전용 |

#### 분석

| 카테고리 | 분석 |
|----------|------|
| **🟡 정리** | `screenshots = "0.8.10"` — 소스 코드에서 **사용되지 않음**. 빌드 시간 증가 원인 |
| **🟡 안정성** | `ort = "2.0.0-rc.11"` — Release Candidate 버전. 프로덕션 배포 시 안정 버전 확인 필요 |
| **🟢 설계** | `crate-type = ["staticlib", "cdylib", "rlib"]` — Tauri 2 권장 패턴 ✅ |
| **🟢 설계** | `[target.'cfg(not(...))'.dependencies]` — 모바일 제외 조건부 의존성 ✅ |
| **🟡 정리** | `lazy_static` — 최신 Rust에서는 `std::sync::OnceLock` 또는 `std::sync::LazyLock`으로 대체 가능 |

---

### 1.2 `tauri.conf.json` (51줄) — Tauri 설정

| 카테고리 | 분석 |
|----------|------|
| **✅ 보안** | `"csp": "default-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:* ..."` — CSP 활성화됨. 다만 `unsafe-inline`, `unsafe-eval` 사용으로 완전한 보호는 아님. 프로덕션 배포 시 강화 권장 |
| **✅ 일관성** | `"identifier": "com.force-focus.app"` — 제품명과 일치하도록 수정됨 |
| **🟡 일관성** | `"title": "desktop-agent"` — 제품명(`"productName": "Force-Focus"`)과 불일치 |
| **🟢 설계** | `"visible": false` — 시작 시 숨김 (트레이 앱) ✅ |
| **🟢 설계** | `"resources": ["resources/models/*"]` — ML 모델 번들 ✅ |
| **🟢 설계** | Deep Link 스킴 `"force-focus"` ✅ |

---

### 1.3 `vite.config.ts` (45줄) — Vite 빌드

| 카테고리 | 분석 |
|----------|------|
| **🟢 설계** | Multi-entry: `main` (index.html), `overlay` (overlay.html), `widget` (widget.html) ✅ |
| **🟢 설계** | `strictPort: true` + `port: 1420` — Tauri 연동 ✅ |
| **🟢 설계** | `ignored: ["**/src-tauri/**"]` — Rust 변경 시 HMR 방지 ✅ |

---

### 1.4 `tsconfig.json` (26줄) — TypeScript 설정

| 카테고리 | 분석 |
|----------|------|
| **🟢 설계** | `"strict": true` ✅ |
| **🟢 설계** | `"noUnusedLocals": true`, `"noUnusedParameters": true` ✅ |
| **🟢 설계** | `"target": "ES2020"` — 모던 브라우저/Tauri 호환 ✅ |

---

### 1.5 `package.json` (41줄) — npm 설정

| 카테고리 | 분석 |
|----------|------|
| **🟡 분류** | `msw`(L31)가 `devDependencies`에 있고 `"msw"` 설정(L35-39)이 루트에 존재. Phase 4에서 MSW 코드는 삭제되었으므로 `msw` 의존성과 설정 제거 권장 |
| **🟢 설계** | Tauri 플러그인 JS 바인딩 (`@tauri-apps/plugin-*`) 모두 `dependencies`에 포함 ✅ |
| **🟡 정리** | `react-icons`(L21) — Phase 4에서 MSW 기반 MainView 삭제 후 실사용 여부 재확인 필요. `MainView.tsx`(새 버전)에서 사용 중이면 유지, 아니면 삭제 | |

---

### 1.6 `tests/integration_test.rs` (69줄) — 통합 테스트

| 카테고리 | 분석 |
|----------|------|
| **🟢 설계** | FSM 파이프라인 검증: 30초 StrongOutlier → DRIFT + TriggerNotification → 60초 → DISTRACTED + TriggerOverlay ✅ |
| **🟡 정리** | L3-4 `WindowInfo`, `InputStats` import — **미사용** (dead import) |
| **🟡 커버리지** | 테스트 1개만 존재. Inlier 복귀, 수동 리셋, 경계값 등 추가 테스트 권장 |

---

## 2. 발견 사항 요약

### 🔴 높은 우선순위

| # | 파일 | 이슈 | 상태 |
|---|------|------|------|
| C-1 | tauri.conf.json | `"csp"` 에 `unsafe-inline`, `unsafe-eval` 포함 — 프로덕션 강화 필요 | ✅ 부분 수정 (null→정책 적용) |

### 🟡 중간 우선순위

| # | 파일 | 이슈 |
|---|------|------|
| C-2 | Cargo.toml | `screenshots` 크레이트 미사용 | ⏳ |
| C-3 | Cargo.toml | `ort` RC 버전 사용 | ⏳ |
| C-4 | tauri.conf.json | title ≠ productName 불일치 | ⏳ |
| C-5 | integration_test.rs | dead imports (`WindowInfo`, `InputStats`) | ⏳ |
| C-6 | integration_test.rs | 테스트 커버리지 부족 (1개) | ⏳ |
| C-9 | package.json | `msw` devDeps + 설정 잔존 (Phase 4에서 코드 삭제되었으므로 제거 권장) | ⏳ |

### 🟢 낮은 우선순위

| # | 파일 | 이슈 |
|---|------|------|
| C-7 | Cargo.toml | `lazy_static` → `OnceLock` 전환 권장 |
| C-8 | package.json | `react-icons` 실사용 확인 필요 |
