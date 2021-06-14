#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';
import { AwsCdkWithRustStack } from '../lib/aws-cdk-with-rust-stack';

const app = new cdk.App();
new AwsCdkWithRustStack(app, 'AwsCdkWithRustStack');
