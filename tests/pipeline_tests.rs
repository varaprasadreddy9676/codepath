use ai_platform::{interpreter, context, evidence, evaluator, composer};

#[tokio::test]
async fn test_full_pipeline_routing() {
    let raw_text = "Can you explain the bill status is still draft? Let me know about BILL-1234".to_string();
    
    // Test native intent and Regex
    let request = interpreter::interpret_intent(raw_text).await;
    assert_eq!(request.intent, "state_transition_explanation");
    assert_eq!(request.entity_type, Some("bill".to_string()));
    assert_eq!(request.identifiers.get("extracted_id").unwrap(), "BILL-1234");

    // Test Context Routing
    let context_pkg = context::resolve_context(&request).await;
    assert!(context_pkg.probable_modules.contains(&"workflow_engine".to_string()));
    assert!(context_pkg.relevant_code_nodes.len() > 0);

    // Test DB Connections and State Extraction Integration
    let evidence_pkg = evidence::collect_evidence(&context_pkg).await;
    assert!(evidence_pkg.db_evidence.is_some());

    // Test Evaluation mathematics
    let result = evaluator::evaluate_constraints(&evidence_pkg).await;
    assert!(result.confidence > 0.95);

    // Test the Composer formatter
    let explanation = composer::compose_explanation(&result).await;
    assert!(explanation.contains("DIAGNOSTIC REPORT"));
    assert!(explanation.contains("discount_enabled"));
}
