// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT-0

use dynamodb::model::AttributeValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FitnessScore {
    username: String,
    version: String,
    age: u32,
    score: u32,
}

impl FitnessScore {
    pub fn new(username: String, version: String, age: u32, score: u32) -> Self {
        FitnessScore {
            username,
            version,
            age,
            score,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn age(&self) -> &u32 {
        &self.age
    }

    pub fn score(&self) -> &u32 {
        &self.score
    }

    pub fn attr_username(&self) -> AttributeValue {
        AttributeValue::S(self.username().to_string())
    }

    pub fn attr_version(&self) -> AttributeValue {
        AttributeValue::S(self.version().to_string())
    }

    pub fn attr_age(&self) -> AttributeValue {
        AttributeValue::N(self.age().to_string())
    }

    pub fn attr_score(&self) -> AttributeValue {
        AttributeValue::N(self.score().to_string())
    }
}
