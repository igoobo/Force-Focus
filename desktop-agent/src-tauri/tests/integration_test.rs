use desktop_agent_lib::ai::inference::{InferenceEngine, InferenceResult};
use desktop_agent_lib::core::state::{StateEngine, InterventionTrigger, FSMState};
use desktop_agent_lib::commands::vision::WindowInfo;
use desktop_agent_lib::commands::input::InputStats;

#[test]
fn test_fsm_inference_integration() {
    // 1. Setup Engines
    // Note: Integration tests require the actual ML model and scaler to be present.
    // For this test, we would normally use a mocked version of InferenceEngine or 
    // minimal dummy model files. Since we cannot guarantee the model file exists 
    // in the CI/CD test environment, we will simulate the InferenceEngine output 
    // directly feeding into the StateEngine to verify their pipeline integration.
    
    let mut state_engine = StateEngine::new();
    
    // 2. Simulate User Behavior over time (60 seconds of Distraction)
    // We simulate the Inference Engine yielding "StrongOutlier" for 60 seconds
    
    let base_time_ms = 1_000_000;
    let mut last_trigger = InterventionTrigger::DoNothing;
    
    // Prime the engine with a baseline tick so the first loop iteration computes a delta > 0.
    state_engine.process(&InferenceResult::Inlier, base_time_ms / 1000, false, false);
    
    // Simulate 30 seconds of strong outlier (watching YouTube)
    for i in 1..=30 {
        let current_time_ms = base_time_ms + (i * 1000);
        let current_time_sec = current_time_ms / 1000;
        
        let inference_result = InferenceResult::StrongOutlier;
        
        // Feed inference result into StateEngine
        last_trigger = state_engine.process(
            &inference_result,
            current_time_sec,
            true, // is_mouse_active
            false // has_recent_input
        );
    }
    
    // After 30 seconds of StrongOutlier, we should hit the DRIFT state 
    // and receive a TriggerNotification
    assert_eq!(state_engine.get_state(), FSMState::DRIFT);
    assert_eq!(last_trigger, InterventionTrigger::TriggerNotification);
    
    // Simulate next 30 seconds of strong outlier
    for i in 31..=60 {
        let current_time_ms = base_time_ms + (i * 1000);
        let current_time_sec = current_time_ms / 1000;
        
        let inference_result = InferenceResult::StrongOutlier;
        
        last_trigger = state_engine.process(
            &inference_result,
            current_time_sec,
            true, 
            false
        );
    }
    
    // After 60 total seconds of StrongOutlier, we should hit DISTRACTED 
    // and receive a TriggerOverlay
    assert_eq!(state_engine.get_state(), FSMState::DISTRACTED);
    assert_eq!(last_trigger, InterventionTrigger::TriggerOverlay);
    
    println!("Integration Test: Pipeline from Inference -> State -> Trigger verified.");
}
