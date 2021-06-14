// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT-0

use dynamodb::model::AttributeValue;
use fitness_score_def::FitnessScore;
use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request, RequestExt, Response,
};
use serde::Serialize;
use thiserror::Error;

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize)]
struct FitnessScoreGetResponse {
    fitness_scores: Vec<FitnessScore>,
}

#[derive(Serialize)]
struct FitnessScoreGetErrorResponse {
    #[serde(rename(serialize = "type"))]
    ty: String,
    message: String,
}

#[derive(Debug, Error)]
enum FitnessScoreGetError {
    #[error("version is not specified")]
    VersionNotSpecified,
    #[error("unknown sort type (`{0}`)")]
    UnknownSortType(String),
    #[error("unknown order type (`{0}`)")]
    UnknownOrderType(String),
    #[error("error during querying dynamodb")]
    DynamoDBQueryError,
}

impl IntoResponse for FitnessScoreGetError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            FitnessScoreGetError::DynamoDBQueryError => 500,
            _ => 400,
        };

        let resp = FitnessScoreGetErrorResponse {
            ty: format!("{:#?}", self),
            message: format!("{:}", self),
        };

        Response::builder()
            .status(status)
            .body(Body::from(serde_json::to_string(&resp).unwrap()))
            .expect("failed to render result")
    }
}

fn attribute_value_to_string(value: &AttributeValue) -> String {
    match value {
        AttributeValue::S(s) => s.to_string(),
        _ => unreachable!(),
    }
}

fn attribute_value_to_u32(value: &AttributeValue) -> u32 {
    match value {
        AttributeValue::N(n) => n.parse().unwrap(),
        _ => unreachable!(),
    }
}

fn dynamodb_query_to_response(
    dynamodb_query_output: dynamodb::output::QueryOutput,
) -> Response<Body> {
    let fitness_scores: Vec<FitnessScore> = dynamodb_query_output
        .items
        .unwrap()
        .into_iter()
        .map(|f| {
            FitnessScore::new(
                attribute_value_to_string(f.get("username").unwrap()),
                attribute_value_to_string(f.get("version").unwrap()),
                attribute_value_to_u32(f.get("age").unwrap()),
                attribute_value_to_u32(f.get("score").unwrap()),
            )
        })
        .collect();

    let resp = FitnessScoreGetResponse { fitness_scores };

    Response::builder()
        .status(200)
        .body(Body::from(serde_json::to_string(&resp).unwrap()))
        .expect("failed to render result")
}

#[tokio::main]
async fn main() -> Result<(), StdError> {
    lambda_runtime::run(handler(func)).await?;
    Ok(())
}

async fn func(event: Request, _: Context) -> Result<impl IntoResponse, StdError> {
    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME is not set");

    let sort = event
        .query_string_parameters()
        .get("sort")
        .map(String::from)
        .unwrap_or("score".to_string());

    let order = match event
        .query_string_parameters()
        .get("order")
        .unwrap_or("asc")
    {
        "asc" => true,
        "desc" => false,
        unknown => {
            return Ok(FitnessScoreGetError::UnknownOrderType(unknown.to_string()).into_response());
        }
    };

    let version = if let Some(v) = event.query_string_parameters().get("version") {
        v.to_string()
    } else {
        return Ok(FitnessScoreGetError::VersionNotSpecified.into_response());
    };

    match sort.as_ref() {
        "score" => {
            let index_name =
                std::env::var("INDEX_NAME_SCORE").expect("INDEX_NAME_SCORE is not set");

            let min_score = event
                .query_string_parameters()
                .get("min_score")
                .or(Some("-1"))
                .map(String::from)
                .unwrap_or("-1".to_string());

            let max_score = event
                .query_string_parameters()
                .get("max_score")
                .or(Some("101"))
                .map(String::from)
                .unwrap_or("101".to_string());

            dynamodb::Client::from_env()
                .query()
                .table_name(table_name)
                .index_name(index_name)
                .key_condition_expression(
                    "#version = :version and #score BETWEEN :min_score AND :max_score",
                )
                .expression_attribute_names("#version", "version")
                .expression_attribute_names("#score", "score")
                .expression_attribute_values(":version", AttributeValue::S(version))
                .expression_attribute_values(":min_score", AttributeValue::N(min_score))
                .expression_attribute_values(":max_score", AttributeValue::N(max_score))
                .scan_index_forward(order)
                .send()
                .await
                .map(dynamodb_query_to_response)
                .map_err(|_| FitnessScoreGetError::DynamoDBQueryError)
                .or_else(|e| Ok(e.into_response()))
        }
        "age" => {
            let index_name = std::env::var("INDEX_NAME_AGE").expect("INDEX_NAME_AGE is not set");

            let min_age = event
                .query_string_parameters()
                .get("min_age")
                .or(Some("-1"))
                .map(String::from)
                .unwrap_or("-1".to_string());

            let max_age = event
                .query_string_parameters()
                .get("max_age")
                .or(Some("200"))
                .map(String::from)
                .unwrap_or("200".to_string());

            dynamodb::Client::from_env()
                .query()
                .table_name(table_name)
                .index_name(index_name)
                .key_condition_expression(
                    "#version = :version and #age BETWEEN :min_age AND :max_age",
                )
                .expression_attribute_names("#version", "version")
                .expression_attribute_names("#age", "age")
                .expression_attribute_values(":version", AttributeValue::S(version))
                .expression_attribute_values(":min_age", AttributeValue::N(min_age))
                .expression_attribute_values(":max_age", AttributeValue::N(max_age))
                .scan_index_forward(order)
                .send()
                .await
                .map(dynamodb_query_to_response)
                .map_err(|_| FitnessScoreGetError::DynamoDBQueryError)
                .or_else(|e| Ok(e.into_response()))
        }
        unknown => Ok(FitnessScoreGetError::UnknownSortType(unknown.to_string()).into_response()),
    }
}
