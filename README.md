# AWS CDK with Rust

This project shows you how to use [Rust](https://www.rust-lang.org/) in Lambda function and how to deploy them by [AWS CDK](https://aws.amazon.com/jp/cdk/).

The stack will deploy
- Amazon API Gateway
- Amazon DynamoDB
- AWS Lambda functions written in Rust.
  - It uses [aws-lambda-rust-runtime](https://github.com/awslabs/aws-lambda-rust-runtime) and [aws-sdk-rust](https://github.com/awslabs/aws-sdk-rust).

## Pre-requirements

- docker ([Instrallation Guide](https://docs.docker.com/get-docker/))
- node ([Installation Guide](https://nodejs.org/ja/download/))
- cdk ([Installation Guide](https://docs.aws.amazon.com/cdk/latest/guide/getting_started.html))

For the deployment, rust toolchain is not required since we use docker as a builder.

## Deployment

Before the cdk deployment, you need to create assets of rust projects for lambda functions.
Use the build script at [`build.sh`](/build.sh).
```bash
./bulid.sh
```

Next, deploy the cdk stack. Read the [Getting Started](https://docs.aws.amazon.com/cdk/latest/guide/getting_started.html) guide if you need.
```bash
# If you never bootstrap your AWS account for cdk, you need to do it.
cdk bootstrap

# Intall the dependencies
npm install

# Deploy it!
cdk deploy
```

## Check the API

We deployed "Fitness Training Score" API, that is, store the latset score of the training for each users.
The storing data structure have username (string), version (string), age (integer), and score (integer) as attributes.

Let's store the first score.
```bash
# Replace <API_ID> and <REGION>
export API_ENDPOINT=https://<API_ID>.execute-api.<REGION>.amazonaws.com/prod

curl \
    -v -XPOST \
    -d '{"username":"user0","version":"0","age":23,"score":87}' \
    -H "Content-Type:application/json" \
    ${API_ENDPOINT}/fitness
```

You will get `{"success":true}` if it's success.
The Amazon DynamoDB database have the username as a partitionKey and the version as a sortKey.
Basically the version is fixed "0".
So you need to change username to store other data.
Please add other data with changing username, age and score.

Let's query the results you stored.
You can specify version, sort, order, min_score, max_score, min_age, and max_age where
- the version is the training version that is fixed "0".
- the sort can specify the sort key that is one of the "score" or "age". (the default is "score".)
- the order can specify the order that is one of "desc" or "asc". (the default is "asc".)
- min_* and max_* can specify the range.
  - min_age and max_age will be ignored if you specify `sort=score`.
  - min_score and max_score will be ignored if you specify `sort=age`.

For example, the query below will get "the results of the training version 0 sorted by the score and ordered by desc. The score range is from 70 to 100."
```bash
curl -v "${API_ENDPOINT}/fitness?version=0&sort=score&order=desc&min_score=70&max_score=100"
```

## About Rust projects structure

This section describes how to construct the rust projects and [`lambda`](/lambda) directory.
We use [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to create multiple binaries.
See [`lambda/Cargo.toml`](/lambda/Cargo.toml)
Alternatively, we can use [`[[bin]]`](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries) section to create multiple binaries in one project.

[`fitness-score-def`](/lambda/fitness-score-def) is a common structure definition that is used in [`fitness-score-get`](/lambda/fitness-score-get) and [`fitness-score-store`](/lambda/fitness-score-store).
It doesn't create a binary since it's a library. [`fitness-score-get`](/lambda/fitness-score-get) and [`fitness-score-store`](/lambda/fitness-score-store) will be the AWS Lambda functions.

## Clean

Execute below.
```bash
cdk destroy
```
