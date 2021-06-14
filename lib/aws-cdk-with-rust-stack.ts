// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT-0
import * as cdk from '@aws-cdk/core';
import * as lambda from '@aws-cdk/aws-lambda';
import * as s3Assets from '@aws-cdk/aws-s3-assets';
import * as apigateway from '@aws-cdk/aws-apigateway';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import * as path from 'path';

export class AwsCdkWithRustStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const fitnessScoreTable = new dynamodb.Table(this, 'AwsCdkWithRustFitnessScoreTable', {
      partitionKey: {
        name: 'username',
        type: dynamodb.AttributeType.STRING,
      },
      sortKey: {
        name: 'version',
        type: dynamodb.AttributeType.STRING,
      },
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
    });

    const indexNameAge = 'FitnessScoreSortByAge';
    const indexNameScore = 'FitnessScoreSortByScore';

    fitnessScoreTable.addGlobalSecondaryIndex({
      indexName: indexNameAge,
      partitionKey: {
        name: 'version',
        type: dynamodb.AttributeType.STRING,
      },
      sortKey: {
        name: 'age',
        type: dynamodb.AttributeType.NUMBER,
      },
    });

    fitnessScoreTable.addGlobalSecondaryIndex({
      indexName: indexNameScore,
      partitionKey: {
        name: 'version',
        type: dynamodb.AttributeType.STRING,
      },
      sortKey: {
        name: 'score',
        type: dynamodb.AttributeType.NUMBER,
      },
    });

    const lambdaCommonOptions: lambda.FunctionOptions = {
      environment: {
        TABLE_NAME: fitnessScoreTable.tableName,
        INDEX_NAME_AGE: indexNameAge,
        INDEX_NAME_SCORE: indexNameScore,
      }
    };

    const fitnessScoreGet = this.createNewFunction('fitness_score_get', lambdaCommonOptions);
    const fitnessScoreStore = this.createNewFunction('fitness_score_store', lambdaCommonOptions);

    fitnessScoreTable.grantReadData(fitnessScoreGet);
    fitnessScoreTable.grantWriteData(fitnessScoreStore);

    const api = new apigateway.RestApi(this, 'AwsCdkWithRustApi');
    const fitnessScoreEndpoint = api.root.addResource('fitness');

    fitnessScoreEndpoint.addMethod('GET', new apigateway.LambdaIntegration(fitnessScoreGet));
    fitnessScoreEndpoint.addMethod('POST', new apigateway.LambdaIntegration(fitnessScoreStore));
  }

  createNewFunction(name: string, props?: lambda.FunctionOptions): lambda.Function {
    const asset = new s3Assets.Asset(this, `AwsCdkWithRustAsset_${name}`, {
      path: path.join(__dirname, '..', 'lambda', `${name}.zip`),
    });

    const func = new lambda.Function(this, `AwsCdkWithRustFunction_${name}`, {
      runtime: lambda.Runtime.PROVIDED,
      code: lambda.Code.fromBucket(asset.bucket, asset.s3ObjectKey),
      handler: 'main', // It's ignored
      ...props,
    });

    return func;
  }
}
