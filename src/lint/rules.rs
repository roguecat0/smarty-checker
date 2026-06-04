mod balanced_html_in_control_flow;
mod no_smarty_in_script;

use std::collections::HashSet;
use std::fmt;

use crate::config::Config;
use crate::lint::Rule;

pub use balanced_html_in_control_flow::BalancedHtmlInControlFlow;
pub use no_smarty_in_script::{NoControlFlowInScript, NoSmartyInScript};

struct RuleSpec {
    id: &'static str,
    default_enabled: bool,
    create: fn() -> Box<dyn Rule>,
}

const RULE_SPECS: &[RuleSpec] = &[
    RuleSpec {
        id: "smarty/balanced-html-in-control-flow",
        default_enabled: true,
        create: || Box::new(BalancedHtmlInControlFlow),
    },
    RuleSpec {
        id: "smarty/no-control-flow-in-script",
        default_enabled: true,
        create: || Box::new(NoControlFlowInScript),
    },
    RuleSpec {
        id: "smarty/no-smarty-in-script",
        default_enabled: false,
        create: || Box::new(NoSmartyInScript),
    },
];

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    RULE_SPECS
        .iter()
        .filter(|spec| spec.default_enabled)
        .map(|spec| (spec.create)())
        .collect()
}

pub fn configured_rules(config: &Config) -> Result<Vec<Box<dyn Rule>>, RuleConfigError> {
    validate_config(config)?;

    Ok(RULE_SPECS
        .iter()
        .filter(|spec| config.is_rule_enabled(spec.id, spec.default_enabled))
        .map(|spec| (spec.create)())
        .collect())
}

pub fn validate_config(config: &Config) -> Result<(), RuleConfigError> {
    let known_rule_ids: HashSet<&str> = RULE_SPECS.iter().map(|spec| spec.id).collect();

    let mut unknown_rule_ids: Vec<String> = config
        .overrides()
        .keys()
        .filter(|rule_id| !known_rule_ids.contains(rule_id.as_str()))
        .cloned()
        .collect();
    unknown_rule_ids.sort();

    if unknown_rule_ids.is_empty() {
        Ok(())
    } else {
        Err(RuleConfigError::UnknownRuleIds(unknown_rule_ids))
    }
}

#[derive(Debug)]
pub enum RuleConfigError {
    UnknownRuleIds(Vec<String>),
}

impl fmt::Display for RuleConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleConfigError::UnknownRuleIds(rule_ids) => {
                write!(f, "unknown rule id")?;
                if rule_ids.len() != 1 {
                    write!(f, "s")?;
                }
                write!(f, ": {}", rule_ids.join(", "))
            }
        }
    }
}

impl std::error::Error for RuleConfigError {}

#[cfg(test)]
mod config_tests {
    use super::{RuleConfigError, configured_rules, default_rules};
    use crate::config::Config;

    #[test]
    fn default_rules_keep_the_existing_rule_set() {
        let rules = default_rules();

        let rule_ids: Vec<&str> = rules.iter().map(|rule| rule.id()).collect();
        assert_eq!(
            rule_ids,
            vec![
                "smarty/balanced-html-in-control-flow",
                "smarty/no-control-flow-in-script"
            ]
        );
    }

    #[test]
    fn configured_rules_can_enable_and_disable_rules() {
        let config = Config::from_toml_for_test(
            r#"
            [rules]
            "smarty/balanced-html-in-control-flow" = false
            "smarty/no-smarty-in-script" = true
            "#,
        );

        let rules = configured_rules(&config).expect("rules");

        let rule_ids: Vec<&str> = rules.iter().map(|rule| rule.id()).collect();
        assert_eq!(
            rule_ids,
            vec![
                "smarty/no-control-flow-in-script",
                "smarty/no-smarty-in-script"
            ]
        );
    }

    #[test]
    fn configured_rules_reject_unknown_rules() {
        let config = Config::from_toml_for_test(
            r#"
            [rules]
            "smarty/typo" = true
            "#,
        );

        let error = match configured_rules(&config) {
            Ok(_) => panic!("unknown rule should fail"),
            Err(error) => error,
        };

        assert!(matches!(error, RuleConfigError::UnknownRuleIds(_)));
        assert_eq!(error.to_string(), "unknown rule id: smarty/typo");
    }
}

#[cfg(test)]
pub fn run_rule_for_test(
    rule: &dyn Rule,
    ctx: &crate::lint::LintContext<'_>,
) -> Vec<crate::diagnostic::Diagnostic> {
    let mut diagnostics = Vec::new();
    rule.check(ctx, &mut diagnostics);
    diagnostics
}
