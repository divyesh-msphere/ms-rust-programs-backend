use crate::adapters::generated::grpc_services::buffa::programs::v1::{
    BatchUpdateFieldValuesRequest, BatchUpdateFieldValuesResponse, FieldValue as GrpcFieldValue,
    ListFieldValuesRequest, ListFieldValuesResponse,
};
use crate::adapters::inputs::grpc_apis::shared::json_struct::{buffa_value_to_json, json_to_buffa_value};
use crate::adapters::inputs::shared::parse;
use crate::application::errors::application_error::ApplicationError;
use crate::application::mappers::grpc_api_mapper::GrpcApiMapper;
use crate::application::usecases::program_field_value::batch_update_field_values::{
    BatchUpdateFieldValuesInput, BatchUpdateFieldValuesOutput,
};
use crate::application::usecases::program_field_value::list_field_values::{
    ListFieldValuesInput, ListFieldValuesOutput,
};
use crate::application::utils::datetime::format_datetime;
use crate::domain::aep::ResourceIdentifier;
use crate::domain::program_field_value::{StoredValue, ValueInput};

fn to_grpc_field_value(value: StoredValue) -> GrpcFieldValue {
    GrpcFieldValue {
        setting_id: value.setting_id.to_string(),
        module_key: value.module_key,
        key: value.key,
        name: value.name,
        r#type: value.field_type.as_str().to_string(),
        value: value.value.map(json_to_buffa_value).into(),
        option_ids: value.option_ids.into_iter().map(|id| id.to_string()).collect(),
        update_time: format_datetime(value.update_time),
        ..Default::default()
    }
}

pub struct ListFieldValuesGrpcMapper;

impl GrpcApiMapper<ListFieldValuesOutput, ListFieldValuesInput, ListFieldValuesResponse, ListFieldValuesRequest>
    for ListFieldValuesGrpcMapper
{
    fn input(request: ListFieldValuesRequest) -> Result<ListFieldValuesInput, ApplicationError> {
        Ok(ListFieldValuesInput {
            program: ResourceIdentifier::parse(&request.program),
            module_key: request.module.map(|m| m.trim().to_string()).filter(|m| !m.is_empty()),
        })
    }

    fn output_success(output: ListFieldValuesOutput) -> ListFieldValuesResponse {
        ListFieldValuesResponse {
            version: output.version,
            results: output.values.into_iter().map(to_grpc_field_value).collect(),
            ..Default::default()
        }
    }
}

pub struct BatchUpdateFieldValuesGrpcMapper;

impl BatchUpdateFieldValuesGrpcMapper {
    pub fn input(request: BatchUpdateFieldValuesRequest, actor: String) -> Result<BatchUpdateFieldValuesInput, ApplicationError> {
        let mut values = Vec::with_capacity(request.values.len());
        for update in request.values {
            let id = parse::uuid("values.setting_id", &update.setting_id)?;
            let input = if update.clear {
                ValueInput::default()
            } else {
                ValueInput {
                    value: update.value.into_option().map(buffa_value_to_json),
                    option_ids: (!update.option_ids.is_empty())
                        .then(|| update.option_ids.iter().map(|raw| parse::uuid("values.option_ids", raw)).collect::<Result<Vec<_>, _>>())
                        .transpose()?,
                }
            };
            values.push((id, input));
        }
        Ok(BatchUpdateFieldValuesInput {
            program: ResourceIdentifier::parse(&request.program),
            version: request.version,
            values,
            actor,
        })
    }

    pub fn output_success(output: BatchUpdateFieldValuesOutput) -> BatchUpdateFieldValuesResponse {
        BatchUpdateFieldValuesResponse {
            version: output.version,
            changed: i32::try_from(output.changed).unwrap_or(i32::MAX),
            results: output.values.into_iter().map(to_grpc_field_value).collect(),
            ..Default::default()
        }
    }
}
