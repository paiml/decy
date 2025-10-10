#!/bin/bash
# Test script to see current Box generation output

cargo test --package decy-codegen test_transform_variable_declaration_with_malloc -- --nocapture 2>&1 | grep -A 20 "test_transform"
