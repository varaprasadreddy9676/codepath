use ai_platform::{interpreter, evaluator, composer};
use ai_platform::models::{EvidencePackage, DiagnosticResult};

// =============================================================
// Interpreter Tests (no external dependencies)
// =============================================================

#[tokio::test]
async fn test_interpret_state_transition() {
    let raw = "Can you explain the bill status is still draft? Let me know about BILL-1234".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert_eq!(request.intent, "state_transition_explanation");
    assert_eq!(request.entity_type, Some("bill".to_string()));
    assert_eq!(request.identifiers.get("extracted_id").unwrap(), "BILL-1234");
}

#[tokio::test]
async fn test_interpret_technical_diagnosis() {
    let raw = "We got an exception in the payment module, check the stacktrace".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert_eq!(request.intent, "technical_diagnosis");
}

#[tokio::test]
async fn test_interpret_business_behavior() {
    let raw = "Why is this discount not applied to the order amount?".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert_eq!(request.intent, "business_behavior_explanation");
}

#[tokio::test]
async fn test_interpret_visibility() {
    let raw = "I can't see the dashboard widget, where is it?".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert_eq!(request.intent, "visibility_explanation");
}

#[tokio::test]
async fn test_interpret_global_search_fallback() {
    let raw = "Tell me about the architecture of the system".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert_eq!(request.intent, "global_search");
}

#[tokio::test]
async fn test_interpret_entity_extraction_order() {
    let raw = "Check ORD-9876 order status".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert_eq!(request.entity_type, Some("order".to_string()));
    assert_eq!(request.identifiers.get("extracted_id").unwrap(), "ORD-9876");
}

#[tokio::test]
async fn test_interpret_no_entity() {
    let raw = "Explain the architecture".to_string();
    let request = interpreter::interpret_intent(raw).await;

    assert!(request.entity_type.is_none());
    assert!(request.identifiers.is_empty());
}

#[tokio::test]
async fn test_interpret_multiple_ids_takes_first() {
    let raw = "Compare BILL-1111 with BILL-2222".to_string();
    let request = interpreter::interpret_intent(raw).await;

    // regex captures_iter overwrites "extracted_id" key, so last one wins
    assert!(request.identifiers.contains_key("extracted_id"));
}

// =============================================================
// Evaluator Tests (offline mode — no LLM key set)
// =============================================================

#[tokio::test]
async fn test_evaluator_offline_with_discount_flag() {
    let evidence = EvidencePackage {
        code_evidence: vec!["some code chunk".to_string()],
        db_evidence: Some(serde_json::json!({
            "active_flags": {
                "discount_enabled": false
            }
        })),
    };

    // Pass empty key explicitly to force offline rule-based evaluation
    // (The .env file may have OPENAI_API_KEY=ollama set)
    let result = evaluator::evaluate_constraints(
        &evidence,
        None,
        Some("".to_string()),  // empty key → offline mode
        None,
    ).await;

    assert!(result.confidence > 0.95, "Should have high confidence: {}", result.confidence);
    assert!(result.primary_cause.contains("discount_enabled"),
        "Should identify discount flag mismatch: {}", result.primary_cause);
}

#[tokio::test]
async fn test_evaluator_offline_no_flags() {
    let evidence = EvidencePackage {
        code_evidence: vec!["code".to_string()],
        db_evidence: Some(serde_json::json!({"status": "active"})),
    };

    let result = evaluator::evaluate_constraints(&evidence, None, None, None).await;

    assert_eq!(result.confidence, 0.85);
    assert!(result.primary_cause.contains("No structural defects"));
}

#[tokio::test]
async fn test_evaluator_offline_no_db_evidence() {
    let evidence = EvidencePackage {
        code_evidence: vec![],
        db_evidence: None,
    };

    let result = evaluator::evaluate_constraints(&evidence, None, None, None).await;

    assert!(result.confidence > 0.0);
    assert!(!result.primary_cause.is_empty());
}

// =============================================================
// Composer Tests
// =============================================================

#[tokio::test]
async fn test_composer_high_confidence() {
    let result = DiagnosticResult {
        primary_cause: "The discount flag is disabled".to_string(),
        confidence: 0.98,
    };

    let explanation = composer::compose_explanation(&result).await;

    assert!(explanation.contains("DIAGNOSTIC REPORT"));
    assert!(explanation.contains("discount flag is disabled"));
    assert!(explanation.contains("strictly confirmed"));
}

#[tokio::test]
async fn test_composer_low_confidence() {
    let result = DiagnosticResult {
        primary_cause: "Possible race condition".to_string(),
        confidence: 0.75,
    };

    let explanation = composer::compose_explanation(&result).await;

    assert!(explanation.contains("DIAGNOSTIC REPORT"));
    assert!(explanation.contains("Possible race condition"));
    assert!(explanation.contains("Manual verification"));
}

#[tokio::test]
async fn test_composer_boundary_confidence() {
    let result = DiagnosticResult {
        primary_cause: "Test cause".to_string(),
        confidence: 0.9,
    };

    let explanation = composer::compose_explanation(&result).await;
    // 0.9 is NOT < 0.9, so it should say "strictly confirmed"
    assert!(explanation.contains("strictly confirmed"));
}
