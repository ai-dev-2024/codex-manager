# API Documentation

## Overview

Codex Manager provides an OpenAI-compatible API proxy that intercepts requests and routes them to the appropriate account based on configured strategies.

## Base URL

```
http://127.0.0.1:8080/v1
```

## Authentication

All requests must include an API key header:

```bash
Authorization: Bearer sk-codex-manager
```

The API key is configured in your settings and defaults to `sk-codex-manager`.

## Endpoints

### Health Check

```http
GET /health
```

Returns the health status of the proxy server.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.3.0",
  "accounts": 5,
  "uptime": 3600
}
```

### List Models

```http
GET /v1/models
```

Returns a list of available models across all configured accounts.

**Response:**
```json
{
  "object": "list",
  "data": [
    {
      "id": "gpt-4",
      "object": "model",
      "created": 1687882411,
      "owned_by": "openai"
    },
    {
      "id": "gpt-4-turbo",
      "object": "model",
      "created": 1687882411,
      "owned_by": "openai"
    },
    {
      "id": "gpt-3.5-turbo",
      "object": "model",
      "created": 1677649963,
      "owned_by": "openai"
    }
  ]
}
```

### Chat Completions

```http
POST /v1/chat/completions
```

Creates a chat completion request, routed to the best available account.

**Request Body:**
```json
{
  "model": "gpt-4",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 150,
  "stream": false
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `model` | string | Yes | Model ID to use |
| `messages` | array | Yes | Array of message objects |
| `temperature` | number | No | Sampling temperature (0-2) |
| `max_tokens` | integer | No | Maximum tokens to generate |
| `stream` | boolean | No | Enable streaming response |
| `top_p` | number | No | Nucleus sampling parameter |
| `n` | integer | No | Number of completions |
| `stop` | string/array | No | Stop sequences |
| `presence_penalty` | number | No | Presence penalty (-2 to 2) |
| `frequency_penalty` | number | No | Frequency penalty (-2 to 2) |

**Response (Non-streaming):**
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 9,
    "completion_tokens": 12,
    "total_tokens": 21
  }
}
```

**Streaming Response:**

When `stream: true`, the response uses Server-Sent Events (SSE):

```
data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

### Text Completions

```http
POST /v1/completions
```

Creates a text completion request.

**Request Body:**
```json
{
  "model": "gpt-3.5-turbo-instruct",
  "prompt": "Once upon a time",
  "max_tokens": 100,
  "temperature": 0.7
}
```

**Response:**
```json
{
  "id": "cmpl-123",
  "object": "text_completion",
  "created": 1677652288,
  "model": "gpt-3.5-turbo-instruct",
  "choices": [
    {
      "text": " in a land far away, there lived a brave knight...",
      "index": 0,
      "logprobs": null,
      "finish_reason": "length"
    }
  ],
  "usage": {
    "prompt_tokens": 4,
    "completion_tokens": 16,
    "total_tokens": 20
  }
}
```

### Embeddings

```http
POST /v1/embeddings
```

Creates an embedding vector for the given input.

**Request Body:**
```json
{
  "model": "text-embedding-ada-002",
  "input": "The quick brown fox jumps over the lazy dog"
}
```

**Response:**
```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [0.0023064255, -0.009327292, ...],
      "index": 0
    }
  ],
  "model": "text-embedding-ada-002",
  "usage": {
    "prompt_tokens": 9,
    "total_tokens": 9
  }
}
```

### Image Generation

```http
POST /v1/images/generations
```

Generates images using DALL-E.

**Request Body:**
```json
{
  "model": "dall-e-3",
  "prompt": "A serene mountain landscape at sunset",
  "n": 1,
  "size": "1024x1024",
  "quality": "standard",
  "style": "vivid"
}
```

**Response:**
```json
{
  "created": 1677652288,
  "data": [
    {
      "url": "https://...",
      "revised_prompt": "A peaceful mountain scene..."
    }
  ]
}
```

## Error Responses

All errors follow the OpenAI error format:

```json
{
  "error": {
    "message": "Error description",
    "type": "error_type",
    "param": null,
    "code": "error_code"
  }
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| `invalid_api_key` | The provided API key is invalid |
| `insufficient_quota` | Account has exceeded its quota |
| `rate_limit_exceeded` | Too many requests |
| `invalid_request_error` | Malformed request |
| `server_error` | Internal server error |
| `no_available_accounts` | No accounts available for routing |

### HTTP Status Codes

| Status | Description |
|--------|-------------|
| 200 | Success |
| 400 | Bad Request |
| 401 | Unauthorized |
| 429 | Rate Limited |
| 500 | Internal Server Error |
| 503 | Service Unavailable (no accounts) |

## Routing Headers

Codex Manager adds custom headers to responses for debugging:

| Header | Description |
|--------|-------------|
| `X-Account-ID` | ID of the account used |
| `X-Account-Label` | Label of the account used |
| `X-Routing-Strategy` | Strategy used for routing |
| `X-Retry-Count` | Number of retries attempted |

## Rate Limiting

Codex Manager respects OpenAI's rate limits and implements:

- Per-account rate limit tracking
- Automatic account switching on rate limit errors
- Exponential backoff for retries
- Circuit breaker to prevent hammering failing accounts

## SDK Integration

### Python

```python
import openai

openai.api_key = "sk-codex-manager"
openai.api_base = "http://127.0.0.1:8080/v1"

response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response.choices[0].message.content)
```

### Node.js

```javascript
const { OpenAI } = require('openai');

const openai = new OpenAI({
  apiKey: 'sk-codex-manager',
  baseURL: 'http://127.0.0.1:8080/v1'
});

async function main() {
  const completion = await openai.chat.completions.create({
    model: 'gpt-4',
    messages: [{ role: 'user', content: 'Hello!' }]
  });
  console.log(completion.choices[0].message.content);
}

main();
```

### cURL

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer sk-codex-manager" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## WebSocket Support

WebSocket connections for real-time streaming are supported at:

```
ws://127.0.0.1:8080/v1/stream
```

## Best Practices

1. **Always use streaming** for long completions to reduce latency
2. **Set appropriate timeouts** - account for routing overhead
3. **Handle errors gracefully** - implement retry logic
4. **Monitor headers** - use routing headers for debugging
5. **Use sticky routing** for chat applications to maximize caching
