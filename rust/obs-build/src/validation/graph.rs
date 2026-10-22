//! Declarative native test targets and their execution contracts.
//!
//! Selection is explicit: callers provide platform, feature and dependency
//! facts. An unknown fact is an error, never an implicit reason to drop a test.
use crate::{NativeBuild, SharedLibrary};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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
    /// Include paths and definitions only; compiler policy belongs in the
    /// target's explicit compile_options, without introducing a link edge.
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

impl Test {
    /// Prepare the declared argv/environment without a shell. The runner still
    /// owns timeout, skip and scheduling policy; preparing a command is not a
    /// test result and does not apply those execution policies.
    pub fn command(&self, root: &Path, output: &Path, target: &Path) -> Result<Command, String> {
        let arguments = self
            .command
            .iter()
            .map(|value| expand(value, root, output, target))
            .collect::<Result<Vec<_>, _>>()?;
        let (program, arguments) = arguments
            .split_first()
            .ok_or("native test has no command")?;
        let mut command = Command::new(program);
        command.args(arguments);
        for (key, value) in &self.environment {
            command.env(key, expand(value, root, output, target)?);
        }
        let directory = match &self.working_directory {
            Some(value) => PathBuf::from(expand(value, root, output, target)?),
            None => output.to_owned(),
        };
        command.current_dir(directory);
        Ok(command)
    }
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

    /// Compile selected targets through cc::Build. Named external libraries
    /// must resolve to the caller's selected artifacts, never a source-tree
    /// search or an implicit same-named system library.
    pub fn compile(
        &self,
        build: &NativeBuild,
        root: &Path,
        facts: &BTreeMap<String, bool>,
        libraries: &BTreeMap<String, PathBuf>,
    ) -> Result<BTreeMap<String, PathBuf>, String> {
        let root = dunce::canonicalize(root).map_err(|error| error.to_string())?;
        let selected = self.select(facts)?;
        let mut artifacts = BTreeMap::new();
        for (name, spec) in selected.targets {
            let mut target = SharedLibrary {
                name: name.into(),
                include_dirs: vec![root.clone()],
                compile_options: spec.compile_options.clone(),
                link_options: spec.link_options.clone(),
                ..Default::default()
            };
            for source in &spec.sources {
                let path = root.join(source);
                println!("cargo:rerun-if-changed={}", path.display());
                let extension = if source.ends_with(".c.h") {
                    Some("c")
                } else if source.ends_with(".cpp.hpp") {
                    Some("cpp")
                } else if source.ends_with(".m.h") {
                    Some("m")
                } else if source.ends_with(".mm.hpp") {
                    Some("mm")
                } else {
                    None
                };
                if let Some(extension) = extension {
                    let include = path
                        .to_str()
                        .ok_or("non-UTF8 native test source")?
                        .replace('\\', "/");
                    if include.contains(['\n', '\r', '"']) {
                        return Err(format!("unrepresentable native test source {include:?}"));
                    }
                    let wrapper = build
                        .output
                        .join("native-tests")
                        .join(name)
                        .join(format!("{source}.{extension}"));
                    fs::create_dir_all(
                        wrapper
                            .parent()
                            .ok_or("native test wrapper has no parent")?,
                    )
                    .map_err(|error| error.to_string())?;
                    crate::frontend::write(&wrapper, &format!("#include \"{include}\"\n"))?;
                    target.sources.push(wrapper);
                } else {
                    target.sources.push(path);
                }
            }
            for directory in &spec.includes {
                target.include_dirs.push(root.join(directory));
            }
            for definition in &spec.definitions {
                let (key, value) = definition
                    .split_once('=')
                    .map_or((definition.as_str(), None), |(key, value)| {
                        (key, Some(value.to_owned()))
                    });
                target.definitions.push((key.into(), value));
            }
            for package in &spec.packages {
                crate::libobs::add_package(&mut target, &build.target, package)?;
            }
            for package in &spec.header_packages {
                super::headers_only(&mut target, package)?;
            }
            // Static archives precede their dynamic dependencies under
            // --as-needed, as in the existing native compatibility builder.
            for name in spec
                .archives
                .iter()
                .map(|name| format!("OBS::{name}"))
                .chain(spec.libraries.iter().cloned())
            {
                target.dependencies.push(
                    libraries
                        .get(&name)
                        .ok_or_else(|| format!("unresolved native test library {name}"))?
                        .clone(),
                );
            }
            for flag in &spec.compile_options_if_supported {
                if build
                    .compiler(&target, &build.output, false)
                    .is_flag_supported(flag)
                    .map_err(|error| error.to_string())?
                {
                    target.compile_options.push(flag.clone());
                }
            }
            crate::platform::configure_test_sanitizers(
                build,
                &mut target,
                spec.sanitizers.as_deref(),
            )?;
            artifacts.insert(name.into(), build.executable(&target)?);
        }
        Ok(artifacts)
    }
}

fn expand(value: &str, root: &Path, output: &Path, target: &Path) -> Result<String, String> {
    let mut expanded = String::new();
    let mut rest = value;
    while let Some((prefix, suffix)) = rest.split_once("${") {
        expanded.push_str(prefix);
        let (token, suffix) = suffix
            .split_once('}')
            .ok_or("unterminated native test template")?;
        let path = match token {
            "source" => root,
            "build" => output,
            "target" => target,
            _ => return Err(format!("unknown native test template {token}")),
        };
        expanded.push_str(path.to_str().ok_or("non-UTF8 native test runtime path")?);
        rest = suffix;
    }
    expanded.push_str(rest);
    Ok(expanded)
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
