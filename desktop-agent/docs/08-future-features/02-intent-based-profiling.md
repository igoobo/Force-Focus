# 신규 기능 제안: 사용자 의도 기반 다중 컨텍스트 프로파일링 (Intent-Based Context Profiling)

> **상태**: Proposed (제안됨)
> **적용 대상**: Backend (ML Pipeline, API), Desktop Agent (`ai/inference.rs`, `core/app.rs`)
> **예상 작업 시간**: High

---

## 1. 개요 (Overview)

현재 AI 모델의 핵심 피처인 `context_score`는 앱이나 제목 토큰을 기반으로 `global_map`에서 고정된 점수를 가져옵니다. 
그러나 실제 업무 환경에서 특정 앱이나 웹사이트의 '생산성'은 절대적이지 않습니다. 예를 들어, YouTube는 일반적으로 이탈 요인으로 분류되어 페널티를 받지만, "디자인 레퍼런스 수집" 혹은 "개발 강의 수강" 중일 때 YouTube는 핵심 업무 도구가 됩니다. 

본 제안은 사용자의 현재 **의도(Intent)**나 목표(Goal)에 따라 AI 모델(ONNX)의 가중치 분류 기준이 동적으로 변경되도록 하는 시스템을 구축하는 것입니다.

## 2. 해결하고자 하는 문제점

- **정적 점수의 한계**: `global_map.json`의 고정된 가중치로 인해, 정당한 업무임에도 AI가 이탈(Outlier)로 판단하여 빈번한 False Positive(오탐)를 발생시킬 수 있습니다.
- **임시방편의 한계**: 현재 "나 일하는 중이야" 버튼을 통해 해당 앱을 일시적으로 화이트리스트 처리하는 기능이 있으나, 이는 수동적 조치이며 다른 업무로 넘어갈 때 유연성을 제공하지 못합니다.

## 3. 핵심 기능 동작

1. **인텐트 프리셋 (Intent Presets)**: 사용자가 앱 내에서 "Coding Mode", "Research Mode", "Design Mode" 등의 의도(Intent) 플래그를 설정합니다. (향후 화면을 보고 자동으로 인텐트를 추론하는 기능도 도입 가능).
2. **동적 가중치 스위칭**:
   - Research 모드 에서는 브라우징 활동에 대한 패널티(`WeakOutlier` 등) 감쇄를 적용합니다.
   - Coding 모드 에서는 오직 IDE와 문서 사이트만이 양수(+)의 컨텍스트 스코어를 가집니다.
3. **ML 앙상블 체계**: 백엔드에서는 단일 모델 학습이 아니라, 수집된 로그를 인텐트로 분리(Clustering)하여 **다수의 모드 특화 ONNX 모델**을 생성하여 데스크톱에 배포하거나, 기존 모델에 `intent_vector` 피처 차원을 새롭게 추가합니다.

## 4. 구조적 변경 사항 (Architecture Impact)

- **Backend (`ml/train.py`)**: 학습 시 기존 6차원 피처(`X_context`, `X_log_input` 등)에 무의식적으로 맵핑됐던 정적 스코어 방식을 넘어서, 사용자가 명시한 인텐트를 Group 라벨 삼아 분리 학습을 수행해야 합니다.
- **Desktop Config (`global_map.json`)**: 단일 Key-Value 가 아닌 다중 프로파일 형태(`intent_map.json`)로 확장.
- **Desktop FSM & Core (`inference.rs`)**: 추론 시 현재 사용자의 모드에 매칭되는 Scaler 파라미터나 특정 ONNX를 핫스왑하거나, 추가 피처 차원을 집어넣도록 변경.

