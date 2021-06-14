// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT-0

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
struct FitnessScoreStoreErrorResponse {
    #[serde(rename(serialize = "type"))]
    ty: String,
    message: String,
}

#[derive(Debug, Error)]
enum FitnessScoreStoreError {
    #[error(
        "invalid fitness score payload. it must have username(string), version(string), age(integer), score(integer)"
    )]
    InvalidFitnessScore,
}

impl IntoResponse for FitnessScoreStoreError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            _ => 400,
        };

        let resp = FitnessScoreStoreErrorResponse {
            ty: format!("{:#?}", self),
            message: format!("{:}", self),
        };

        Response::builder()
            .status(status)
            .body(Body::from(serde_json::to_string(&resp).unwrap()))
            .expect("failed to render result")
    }
}

#[tokio::main]
async fn main() -> Result<(), StdError> {
    lambda_runtime::run(handler(func)).await?;
    Ok(())
}

async fn func(event: Request, _: Context) -> Result<impl IntoResponse, StdError> {
    let table_name = std::env::var("TABLE_NAME").expect("TABLE_NAME is not set");

    let fitness_score: FitnessScore =
        if let Some(fitness_score) = event.payload().unwrap_or_else(|_| None) {
            fitness_score
        } else {
            return Ok(FitnessScoreStoreError::InvalidFitnessScore.into_response());
        };

    dynamodb::Client::from_env()
        .put_item()
        .table_name(table_name)
        .item("username", fitness_score.attr_username())
        .item("version", fitness_score.attr_version())
        .item("age", fitness_score.attr_age())
        .item("score", fitness_score.attr_score())
        .send()
        .await?;

    let res = Response::builder()
        .status(200)
        .body(Body::Empty)
        .expect("failed to render response");

    Ok(res)
}
