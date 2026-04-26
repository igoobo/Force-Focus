# 신규 서브 기능 제안: 맥락 유실 방지 및 작업 공간 롤백 (Workspace Snapshot & Restore)

> **상태**: ✅ Implemented (구현 완료)
> **적용 대상**: Desktop Agent (`commands/vision.rs`, `core/app.rs`, Frontend `InterventionOverlay.tsx`)
> **예상 작업 시간**: Medium
> **분류**: 100% Sub Feature (몰입 유지 보조)
> **구현 위치**: `vision.rs:313-364` (restore_workspace), `app.rs:400-410` (snapshot capture), `InterventionOverlay.tsx` (복귀 버튼)

---

## 1. 개요 (Overview)
사용자가 몰입(`FOCUS`) 상태일 때는 IDE와 문서 브라우저 등 업무에 필요한 최적의 창 배치가 형성되어 있습니다. 하지만 잠시 딴짓(`DISTRACTED`)을 하게 되면 SNS, 메신저, 게임 등의 창이 열리면서 화면이 난잡해지고, 다시 원래 업무 화면으로 돌아오기 위해 여러 창을 최소화하거나 닫아야 하는 **'복귀 마찰(Friction)'**이 발생합니다.

본 기능은 사용자가 몰입 상태에 진입할 때 윈도우 창 배치를 메모리에 스냅샷으로 저장해 두고, 이탈 후 발생한 방해 창들을 원클릭으로 최소화하여 원래의 화면 배치로 즉시 롤백해 주는 강력한 보조 기능입니다.

## 2. 해결하고자 하는 문제점
- **인지적 부하 및 귀찮음**: 유튜브나 웹서핑 후 다시 코드나 문서 작업을 하려고 할 때, 딴짓으로 열어둔 창들을 치우는 행위 자체가 귀찮아 복귀를 포기하는 현상 방지.
- **시각적 노이즈 차단**: 원래 업무 배치로 돌아감과 동시에, 시선을 뺏는 다른 창들을 한 번에 최소화(Minimize)하여 시각적 몰입 환경 재구축.

## 3. 핵심 아키텍처 및 동작 파이프라인

### M2 (윈도우 토폴로지 메타데이터) 활용
본 기능은 카메라나 마이크 하드웨어를 사용하지 않으며, 순수 OS 레벨의 윈도우 메타데이터만을 활용합니다.

1. **상태 진입 감지 (Capture Snapshot)**
   - `core/state.rs`: FSM(유한 상태 머신)이 `FOCUS` 상태로 진입하고, 해당 상태가 n초 이상 유지될 때 안정적인 작업 상태로 간주하여 스냅샷 캡처를 트리거합니다.
   - `commands/vision.rs` (또는 유틸): Win32 API (`EnumWindows`, `GetWindowPlacement`)를 호출하여 현재 활성화되고 화면에 띄워져 있는 모든 창의 핸들(HWND), 타이틀, X/Y 좌표 및 크기(Bounds), Z-Order(활성 순서)를 담은 `WorkspaceSnapshot` 구조체를 생성해 `AppCore` 메모리에 캐싱합니다.

2. **개입 발생 (Intervention)**
   - FSM이 `DISTRACTED`로 전이되고 Drift Gauge가 한계에 다다르면 Frontend에 개입 오버레이를 띄웁니다.
   - 이때 Frontend 오버레이 UI에 **"원클릭 작업 복귀 🚀"** 버튼을 노출합니다.

3. **복구 실행 (Restore Execution)**
   - 사용자가 버튼을 클릭하면 Tauri `invoke("restore_workspace")` 커맨드가 백엔드로 호출됩니다.
   - 백엔드는 현재 띄워져 있는 창 중, 스냅샷에 없었던 창(나중에 열린 딴짓 용 앱 등)을 식별하여 Win32 API `ShowWindow(HWND, SW_MINIMIZE)` 로 시야에서 제거합니다.
   - 스냅샷에 저장되어 있던 업무용 창들은 `SetWindowPos`를 통해 원래의 위치와 크기로, 원래의 Z-Order에 맞춰 `SW_RESTORE` 처리합니다.

## 4. 구조적 변경 사항 (Architecture Impact)

- **`core/state.rs`**: 상태 변경 로직 내에 Snapshot 저장/무효화 로직 추가. (예: `AppCore` 구조체 필드에 `last_snapshot: Option<WorkspaceSnapshot>` 추가)
- **`commands/vision.rs`**: 스냅샷 생성 함수 `capture_workspace()` 및 복구 함수 `restore_workspace(snapshot)` 구현 (Win32 API 연동).
- **`frontend/`**: `InterventionOverlay` 컴포넌트 내에 세션 롤백용 UI 버튼 및 Tauri Invoke 핸들러 추가.

