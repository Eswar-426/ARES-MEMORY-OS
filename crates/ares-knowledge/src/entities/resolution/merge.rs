use super::super::models::Entity;
use chrono::Utc;

pub fn merge_entities(target: &mut Entity, source: &Entity) {
    // Keep target name, but merge descriptions if target lacks one
    if target.description.is_none() && source.description.is_some() {
        target.description = source.description.clone();
    }

    // Merge properties
    if let (Some(target_obj), Some(source_obj)) = (
        target.properties.as_object_mut(),
        source.properties.as_object(),
    ) {
        for (k, v) in source_obj {
            if !target_obj.contains_key(k) {
                target_obj.insert(k.clone(), v.clone());
            }
        }
    }

    // Update metadata
    target.updated_at = Utc::now();

    // Boost confidence slightly due to multiple evidence sources
    target.confidence_score = (target.confidence_score + 0.1).min(1.0);
}
