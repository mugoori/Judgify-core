#!/bin/bash

# Load .env file
source .env

# Test OpenAI API key
echo "Testing OpenAI API key..."
echo "Key (first 10 chars): ${OPENAI_API_KEY:0:10}..."

curl https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_tokens": 10
  }' 2>&1 | head -30
