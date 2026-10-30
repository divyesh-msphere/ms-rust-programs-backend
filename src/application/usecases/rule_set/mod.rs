use crate::domain::program::ProgramRepository;
use crate::domain::rule_engine::RuleEngine;
use crate::domain::rule_set::RuleSetRepository;
use std::sync::Arc;

pub mod create_rule_version;
pub mod execute_rule;
pub mod get_rule_set;
pub mod get_rule_version;
pub mod list_rule_sets;
pub mod list_rule_versions;

use create_rule_version::CreateRuleVersionUseCase;
use execute_rule::ExecuteRuleUseCase;
use get_rule_set::GetRuleSetUseCase;
use get_rule_version::GetRuleVersionUseCase;
use list_rule_sets::ListRuleSetsUseCase;
use list_rule_versions::ListRuleVersionsUseCase;

pub struct RuleSetUseCases {
    pub list_rule_sets: ListRuleSetsUseCase,
    pub get_rule_set: GetRuleSetUseCase,
    pub create_rule_version: CreateRuleVersionUseCase,
    pub list_rule_versions: ListRuleVersionsUseCase,
    pub get_rule_version: GetRuleVersionUseCase,
    pub execute_rule: ExecuteRuleUseCase,
}

impl RuleSetUseCases {
    pub fn new(
        program_repository: Arc<dyn ProgramRepository>,
        rule_set_repository: Arc<dyn RuleSetRepository>,
        rule_engine: Arc<dyn RuleEngine>,
    ) -> Self {
        Self {
            list_rule_sets: ListRuleSetsUseCase::new(program_repository.clone(), rule_set_repository.clone()),
            get_rule_set: GetRuleSetUseCase::new(program_repository.clone(), rule_set_repository.clone()),
            create_rule_version: CreateRuleVersionUseCase::new(program_repository.clone(), rule_set_repository.clone()),
            list_rule_versions: ListRuleVersionsUseCase::new(program_repository.clone(), rule_set_repository.clone()),
            get_rule_version: GetRuleVersionUseCase::new(program_repository.clone(), rule_set_repository.clone()),
            execute_rule: ExecuteRuleUseCase::new(program_repository, rule_set_repository, rule_engine),
        }
    }
}
