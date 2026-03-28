use ai_platform::{interpreter, context, evidence, evaluator, composer};

#[tokio::test]
async fn test_full_pipeline_routing() {
    let raw_text = "Why is this bill still draft?".to_string();
    
    // Test Interpreter
    let request = interpreter::interpret_intent(raw_text).await;
    assert_eq!(request.intent, "technical_diagnosis");

    // Test Context Resolver
    let context_pkg = context::resolve_context(&request).await;
    assert_eq!(context_pkg.probable_modules[0], "stub_module");

    // Test Evidence Collector
    let evidence_pkg = evidence::collect_evidence(&context_pkg).await;
    assert!(evidence_pkg.db_evidence.is_none());

    // Test Evaluator
    let result = evaluator::evaluate_constraints(&evidence_pkg).await;
    assert!(result.confidence > 0.9);

    // Test Composer
    let explanation = composer::compose_explanation(&result).await;
    assert!(explanation.contains("Root Cause:"));
}
