//! Declarative native test targets and their execution contracts.
//!
//! Selection is explicit: callers provide platform, feature and dependency
//! facts. An unknown fact is an error, never an implicit reason to drop a test.
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Requirement {
    Constant(bool),
    Fact(String),
    All { all: Vec<Requirement> },
    Any { any: Vec<Requirement> },
    Not { not: Box<Requirement> },
}

impl Default for Requirement {
    fn default() -> Self {
        Self::Constant(true)
    }
}

impl Requirement {
    fn evaluate(&self, facts: &BTreeMap<String, bool>) -> Result<bool, String> {
        match self {
            Self::Constant(value) => Ok(*value),
            Self::Fact(name) => facts
                .get(name)
                .copied()
                .ok_or_else(|| format!("unknown native test requirement {name}")),
            Self::All { all } => {
                let mut selected = true;
                for requirement in all {
                    selected &= requirement.evaluate(facts)?;
                }
                Ok(selected)
            }
            Self::Any { any } => {
                let mut selected = false;
                for requirement in any {
                    selected |= requirement.evaluate(facts)?;
                }
                Ok(selected)
            }
            Self::Not { not } => not.evaluate(facts).map(|value| !value),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub sources: Vec<String>,
    #[serde(default)]
    pub requires: Requirement,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub definitions: Vec<String>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub header_packages: Vec<String>,
    #[serde(default)]
    pub archives: Vec<String>,
    #[serde(default)]
    pub libraries: Vec<String>,
    #[serde(default)]
    pub compile_options: Vec<String>,
    #[serde(default)]
    pub compile_options_if_supported: Vec<String>,
    #[serde(default)]
    pub link_options: Vec<String>,
    #[serde(default)]
    pub sanitizers: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Test {
    pub target: String,
    #[serde(default)]
    pub requires: Requirement,
    #[serde(default = "default_command")]
    pub command: Vec<String>,
    /// None inherits the runner's configured default; Some(0) is unbounded,
    /// matching CTest's explicit TIMEOUT 0 semantics.
    #[serde(default)]
    pub timeout_seconds: Option<u32>,
    #[serde(default)]
    pub skip_return_code: Option<i32>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub resource_locks: Vec<String>,
    #[serde(default)]
    pub run_serial: bool,
}

fn default_command() -> Vec<String> {
    vec!["${target}".into()]
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub schema: u32,
    pub targets: BTreeMap<String, Target>,
    pub tests: BTreeMap<String, Test>,
}

#[derive(Debug)]
pub struct Selection<'a> {
    pub targets: BTreeMap<&'a str, &'a Target>,
    pub tests: BTreeMap<&'a str, &'a Test>,
}

impl Graph {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("native test graph {}: {error}", path.display()))?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let graph: Self = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if graph.schema != 1 {
            return Err(format!(
                "unsupported native test graph schema {}",
                graph.schema
            ));
        }
        if graph.targets.is_empty() || graph.tests.is_empty() {
            return Err("native test graph has no targets or tests".into());
        }
        for (name, target) in &graph.targets {
            validate_name(name)?;
            if target.sources.is_empty() || target.sources.iter().any(String::is_empty) {
                return Err(format!("native test target {name} has no sources"));
            }
        }
        for (name, test) in &graph.tests {
            validate_name(name)?;
            if !graph.targets.contains_key(&test.target) {
                return Err(format!(
                    "native test {name} references absent target {}",
                    test.target
                ));
            }
            if test.command.is_empty() || test.command[0].is_empty() {
                return Err(format!("native test {name} has no command"));
            }
            for value in test
                .command
                .iter()
                .chain(test.environment.values())
                .chain(test.working_directory.iter())
            {
                validate_template(value)?;
            }
        }
        Ok(graph)
    }

    pub fn select(&self, facts: &BTreeMap<String, bool>) -> Result<Selection<'_>, String> {
        let mut selected = Selection {
            targets: BTreeMap::new(),
            tests: BTreeMap::new(),
        };
        for (name, target) in &self.targets {
            if target.requires.evaluate(facts)? {
                selected.targets.insert(name.as_str(), target);
            }
        }
        for (name, test) in &self.tests {
            let enabled = test.requires.evaluate(facts)?;
            if enabled && selected.targets.contains_key(test.target.as_str()) {
                selected.tests.insert(name.as_str(), test);
            }
        }
        Ok(selected)
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.".contains(&byte))
    {
        return Err(format!("invalid native test name {name:?}"));
    }
    Ok(())
}

fn validate_template(mut value: &str) -> Result<(), String> {
    while let Some((_, suffix)) = value.split_once("${") {
        let (token, rest) = suffix
            .split_once('}')
            .ok_or_else(|| format!("unterminated native test template {value:?}"))?;
        if !["source", "build", "target"].contains(&token) {
            return Err(format!("unknown native test template {token}"));
        }
        value = rest;
    }
    Ok(())
}
