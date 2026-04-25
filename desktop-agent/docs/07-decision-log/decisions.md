# 의사결정 기록 (Decision Log)

> **작성일**: 2026-03-21
> **최종 업데이트**: 2026-04-25 (Workspace Snapshot 기능 의사결정 추가)
> **범위**: 코드 리뷰 과정에서 발견된 설계 의사결정 및 수정 근거

---

## 1. 보안: 민감 정보 로그 마스킹

| 항목 | 내용 |
|------|------|
| **기존** | `println!("User logged in: {}", user_email)` 등으로 이메일, 토큰이 콘솔에 노출 |
| **변경** | `[REDACTED]`로 마스킹 |
| **이유** | 콘솔 로그가 로그 파일이나 크래시 리포트에 포함될 수 있으며, 개인정보 보호 규정(GDPR 등) 위반 가능. Deep Link URL의 토큰 노출은 탈취 시 계정 접근 가능 |
| **대안 비교** | ① 로그 레벨 분리 (debug only) — 프로덕션 빌드에서만 안전, ② **마스킹** — 모든 환경에서 안전 ✅, ③ 로그 완전 제거 — 디버깅 불가 |
| **커밋** | `6ecccc6`, `80db381` |

---

## 2. 안정성: `unwrap()` → 안전 패턴

| 항목 | 내용 |
|------|------|
| **기존** | `Mutex::lock().unwrap()` — Mutex poisoning 시 **패닉** (앱 크래시) |
| **변경** | `match lock() { Ok(guard) => guard, Err(_) => return }` 또는 `.map_err()` |
| **이유** | 이벤트 핸들러(tray, widget)에서 패닉 발생 시 전체 앱이 비정상 종료. Mutex poisoning은 드물지만, 다른 스레드에서 panic이 발생한 경우 연쇄 크래시를 방지해야 함 |
| **대안 비교** | ① `unwrap()` 유지 + panic handler — 근본 해결 아님, ② `expect()` + 메시지 — 여전히 패닉, ③ **match/map_err** — graceful degradation ✅ |
| **커밋** | `6ecccc6`, `cb9aa47`, `9df0b7e`, `c7c6741` |

---

## 3. 버그 수정: Command 이중 생성

| 항목 | 내용 |
|------|------|
| **기존** | `schedule.rs` L135에서 `Command::new(&exe_path)` 생성 + 인자 설정 후, L148에서 **새 `Command::new(&exe_path).spawn()`** 으로 실행 → 인자 미적용 |
| **변경** | `cmd.spawn()`으로 변경하여 인자가 설정된 원래 Command로 실행 |
| **이유** | 스케줄된 프로그램 실행 시 인자가 전달되지 않는 **기능 결함** |
| **커밋** | `c7c6741` |

---

## 4. 성능: `FSMState`에 `Copy` trait 추가

| 항목 | 내용 |
|------|------|
| **기존** | `FSMState` enum은 `Clone`만 구현. 비교/전달 시 `.clone()` 필요 |
| **변경** | `#[derive(Copy)]` 추가 |
| **이유** | `FSMState`는 데이터를 보유하지 않는 단순 enum으로, `Copy`가 자연스러움. 불필요한 힙 할당 없이 스택 복사로 전달되어 성능 향상 |
| **커밋** | `6ecccc6` |

---

## 5. 설계 발견: 두 병렬 UI 시스템

| 항목 | 내용 |
|------|------|
| **발견** | `App.tsx`(LoginView, SettingsView)는 Tauri `invoke`를 사용하지만, `MainView`는 `fetch` + MSW를 사용. 두 시스템이 공존 |
| **원인** | 초기 MSW 프로토타입에서 Rust 백엔드 연동으로 전환하는 과정에서 `MainView` 마이그레이션이 미완 |
| **해결** | `api/index.ts`, `mocks/`, `MainView/index.tsx` + 4개 서브컴포넌트 **전체 삭제**. 새 `MainView.tsx`를 Tauri `invoke` 기반으로 재작성 |
| **상태** | ✅ FIXED |

---

## 6. 설계 발견: `feature.rs` Dead Code

| 항목 | 내용 |
|------|------|
| **발견** | `feature.rs`의 `FeatureExtractor`가 정의되어 있지만 실제 사용되지 않음. `app.rs`에서 인라인으로 동일 기능을 구현하되 수학 공식이 불일치 (표준편차 계산, interaction gate 로직) |
| **위험** | 두 구현 중 어느 것이 정확한지 불명확. 향후 유지보수 시 혼란 |
| **해결** |  `feature.rs` **삭제**. `AppCore::start_core_loop()` 인라인 구현을 유일 기준으로 확정 |
| **상태** | ✅ FIXED (커밋 `fe89e11`) |

---

## 7. 보안: OS Keyring → XOR Obfuscation 

| 항목 | 내용 |
|------|------|
| **기존** | `keyring` 크레이트로 OS 자격 증명 관리자에 auth_token 저장 |
| **문제** | Windows 자격 증명 관리자에서 토큰이 증발하는 불안정성 발생. 앱 재시작 시 자동 로그인 실패 |
| **변경** | `keyring` 의존성 제거 → SQLite 내 XOR 난독화 계층으로 대체 |
| **대안 비교** | ① AES 암호화 — 키 관리 복잡성, ② OS keyring 유지 — 불안정, ③ **XOR 난독화** — casual inspection 방지 + 안정성 ✅ |
| **한계** | 키가 바이너리에 하드코딩되어 리버스 엔지니어링으로 복원 가능. 암호화가 아닌 난독화 |
| **상태** | ✅ 구현됨 |

---

## 8. 구조: backend_comm.rs 모듈 분리

| 항목 | 내용 |
|------|------|
| **기존** | `backend_comm.rs` 826줄 단일 파일에 HTTP 클라이언트 + DTO + 10개 Tauri 커맨드 혼재 |
| **변경** | 4개 파일로 분리: `utils/api.rs` (HTTP+DTO), `commands/auth.rs`, `commands/session.rs`, `commands/task.rs` |
| **이유** | SRP(단일 책임 원칙) 위반. 네트워크 계층과 커맨드 핸들러가 결합되어 테스트/유지보수 불리 |
| **커밋** | `fe89e11` |

---

## 9. 정리: MSW 프로토타입 전체 제거

| 항목 | 내용 |
|------|------|
| **기존** | `api/index.ts`, `mocks/browser.ts`, `MainView/index.tsx` + 4개 서브컴포넌트 — MSW 기반 `fetch` 통신 |
| **변경** | 전체 삭제. `MainView.tsx`를 Tauri `invoke` 기반으로 새로 작성 |
| **이유** | MSW 비활성화 상태에서 `MainView`가 동작하지 않음. Tailwind CSS 미설치로 스타일도 미적용. 프론트엔드 15파일 → 15파일 (데드코드 제거 + styles 파일 추가) |
| **상태** | ✅ FIXED  |

---

## 10. 스타일: Tailwind → CSS-in-JS 통일 

| 항목 | 내용 |
|------|------|
| **기존** | LoginView/SettingsView = 인라인 스타일, MainView 서브컴포넌트 = Tailwind CSS 클래스. **혼재** |
| **변경** | CSS-in-JS 패턴(`*.styles.ts`)으로통일. `React.CSSProperties` 객체 사용 |
| **이유** | Tailwind가 설치되지 않은 상태에서 Tailwind 클래스 사용 → 스타일 전혀 미적용 |
| **예외** | `widget_main.tsx`는 아직 인라인 스타일 직접 사용 (styles 파일 미분리) |
| **상태** | ✅ 구현됨 |

---

## 11. 기능: Workspace Snapshot & Restore 

| 항목 | 내용 |
|------|------|
| **문제** | DISTRACTED 상태에서 따짓 창들이 열려 작업 화면이 난잡해지면, 다시 원래 배치로 돌아가는 데 인지적 부하 발생 |
| **결정** | FOCUS 진입 시점에 `WorkspaceSnapshot`(전체 창 HWND+좌표) 캐철 → 개입 시 "작업 복귀" 버튼으로 롤백 |
| **구현** | `vision.rs:restore_workspace` (Win32 `ShowWindow`/`SetWindowPos`) + `app.rs:400-410` (전이 감지) + `InterventionOverlay.tsx` (복귀 버튼) |
| **대안 비교** | ① 가상 데스크톱 — OS 수준 통합 필요, ② 태스크바 그루핑 — 단순 시각적 분류만 가능, ③ **HWND 스냅샷** — 위치+크기+Z-order 까지 복원 가능 ✅ |
| **제한사항** | 메모리 내 캐싱만 지원 (디스크 저장 없음), 앱 재시작 시 스냅샷 소실 |
| **상태** | ✅ 구현됨 |
