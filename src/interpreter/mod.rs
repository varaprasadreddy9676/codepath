use crate::models::InvestigationRequest;
use tracing::info;
use regex::Regex;
use std::collections::HashMap;

pub async fn interpret_intent(raw_text: String) -> InvestigationRequest {
    info!("Interpreting intent from: {}", raw_text);
    
    let mut entity_type = None;
    let mut identifiers = HashMap::new();

    let text_lower = raw_text.to_lowercase();
    
    // Rule-based intent classification mapped directly (resolves legacy build warnings safely)
    let intent = if text_lower.contains("exception") || text_lower.contains("error") || text_lower.contains("stacktrace") {
        "technical_diagnosis".to_string()
    } else if text_lower.contains("why is this") || text_lower.contains("discount") || text_lower.contains("amount") {
        "business_behavior_explanation".to_string()
    } else if text_lower.contains("can't see") || text_lower.contains("where is") {
        "visibility_explanation".to_string()
    } else if text_lower.contains("draft") || text_lower.contains("pending") || text_lower.contains("status") {
        "state_transition_explanation".to_string()
    } else {
        "global_search".to_string()
    };

    // Entity extraction pipeline crossing regex boundaries
    if text_lower.contains("bill") {
        entity_type = Some("bill".to_string());
    } else if text_lower.contains("order") {
        entity_type = Some("order".to_string());
    }

    // Dynamic extraction (e.g., reliably grasping BILL-1234 or ORD-0999)
    if let Ok(re) = Regex::new(r"([A-Z]{3,4}-\d{4,8})") {
        for cap in re.captures_iter(&raw_text) {
            identifiers.insert("extracted_id".to_string(), cap[1].to_string());
        }
    }

    info!("Resolved Syntactical Intent: {} | Found Logical Entity: {:?}", intent, entity_type);

    InvestigationRequest {
        intent,
        original_text: raw_text,
        entity_type,
        identifiers,
    }
}
