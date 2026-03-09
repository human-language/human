use std::collections::BTreeMap;
use serde::Serialize;
use human_resolver::Resolved;
use crate::util::{level_str, value_to_json};

#[derive(Serialize)]
pub struct CompiledOutput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub constraints: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub flows: BTreeMap<String, Vec<String>>,
}

pub fn build(resolved: &Resolved, system_content: Option<&str>) -> CompiledOutput {
    let mut properties = BTreeMap::new();
    for prop in &resolved.agent.properties {
        properties.insert(prop.key.clone(), value_to_json(&prop.value));
    }

    let mut constraints = BTreeMap::new();
    for block in &resolved.constraints {
        let rules: Vec<String> = block
            .constraints
            .iter()
            .map(|c| format!("{} {}", level_str(c.level), c.text))
            .collect();
        constraints.insert(block.name.clone(), rules);
    }

    let mut flows = BTreeMap::new();
    for block in &resolved.flows {
        let steps: Vec<String> = block.steps.iter().map(|s| s.text.clone()).collect();
        flows.insert(block.name.clone(), steps);
    }

    CompiledOutput {
        name: resolved.agent.name.clone(),
        system: system_content.map(|s| s.to_string()),
        properties,
        constraints,
        flows,
    }
}
