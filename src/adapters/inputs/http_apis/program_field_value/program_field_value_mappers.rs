use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::usecases::program_field_value::batch_update_field_values::{
    BatchUpdateFieldValuesInput, BatchUpdateFieldValuesOutput,
};
use crate::application::usecases::program_field_value::list_field_values::{
    ListFieldValuesInput, ListFieldValuesOutput,
};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_field_value::{StoredValue, ValueInput};
use crate::dto::payloads::json::program_field_value::{
    BatchUpdateFieldValuesBodyPayload, BatchUpdateFieldValuesPathPayload, ListFieldValuesPathPayload,
    ListFieldValuesQueryPayload,
};
use crate::dto::presenters::json::program_field_value::{
    BatchUpdateFieldValuesPresenter, FieldValuePresenter, FieldValuesPresenter,
};

fn to_presenter(value: StoredValue) -> FieldValuePresenter {
    FieldValuePresenter {
        setting_id: value.setting_id.to_string(),
        module_key: value.module_key,
        key: value.key,
        name: value.name,
        field_type: value.field_type.as_str().to_string(),
        value: value.value,
        option_ids: value.option_ids.into_iter().map(|id| id.to_string()).collect(),
        update_time: format_datetime(value.update_time),
    }
}

pub struct ListFieldValuesMapper;

impl ListFieldValuesMapper {
    pub fn input(
        path: ListFieldValuesPathPayload,
        query: ListFieldValuesQueryPayload,
    ) -> Result<ListFieldValuesInput, ApplicationError> {
        Ok(ListFieldValuesInput {
            program: ResourceIdentifier::parse(&path.program),
            module_key: query.module.map(|m| m.trim().to_string()).filter(|m| !m.is_empty()),
        })
    }

    pub fn output_success(output: ListFieldValuesOutput) -> FieldValuesPresenter {
        FieldValuesPresenter { version: output.version, results: output.values.into_iter().map(to_presenter).collect() }
    }
}

pub struct BatchUpdateFieldValuesMapper;

impl BatchUpdateFieldValuesMapper {
    pub fn input(
        path: BatchUpdateFieldValuesPathPayload,
        body: BatchUpdateFieldValuesBodyPayload,
        actor: String,
    ) -> Result<BatchUpdateFieldValuesInput, ApplicationError> {
        let mut values = Vec::with_capacity(body.values.len());
        for (setting_id, payload) in body.values {
            let id = parse::uuid("values key (a setting id)", &setting_id)?;
            let input = match payload {
                // `null`, or neither field: clear it.
                None => ValueInput::default(),
                Some(payload) => ValueInput {
                    value: payload.value,
                    option_ids: payload
                        .option_ids
                        .map(|ids| ids.iter().map(|raw| parse::uuid("option_ids", raw)).collect::<Result<Vec<_>, _>>())
                        .transpose()?,
                },
            };
            values.push((id, input));
        }
        Ok(BatchUpdateFieldValuesInput {
            program: ResourceIdentifier::parse(&path.program),
            version: body.version,
            values,
            actor,
        })
    }

    pub fn output_success(output: BatchUpdateFieldValuesOutput) -> BatchUpdateFieldValuesPresenter {
        BatchUpdateFieldValuesPresenter {
            version: output.version,
            changed: output.changed,
            results: output.values.into_iter().map(to_presenter).collect(),
        }
    }
}
