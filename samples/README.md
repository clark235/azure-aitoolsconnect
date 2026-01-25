# Sample Files

This directory contains sample input files and configuration templates for testing Azure AI Services connectivity.

## Sample Input Files

| File | Description | Used By |
|------|-------------|---------|
| `test-speech.wav` | 16kHz, 16-bit, mono PCM audio saying "Hello, this is a connectivity test" | Speech service (stt_short, stt_rest) |
| `test-image.png` | Simple test image for Vision service testing | Vision service (analyze_image, read_text, detect_objects) |
| `test-document.pdf` | Simple PDF document with text | Document Intelligence (layout, read) |

## Configuration Templates

| File | Description |
|------|-------------|
| `config-multiservice.toml` | Configuration for multi-service CognitiveServices resources (S0 tier) |
| `config-dedicated.toml` | Configuration for dedicated service resources (separate keys per service) |

## Quick Start

### Multi-Service Resource

For a multi-service CognitiveServices resource (recommended for testing multiple services):

```bash
# Set environment variables
export AZURE_AI_ENDPOINT="https://your-resource.cognitiveservices.azure.com"
export AZURE_AI_API_KEY="your-api-key"
export AZURE_REGION="swedencentral"

# Test all services
cargo run -- test --services all

# Test specific services
cargo run -- test --services speech,language,vision

# Test with config file
cargo run -- test -c samples/config-multiservice.toml --services all
```

### Command Line Options

```bash
# Test Speech service with custom endpoint
cargo run -- test --services speech \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral

# Test Vision with custom image
cargo run -- test --services vision \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral \
  --input-file samples/test-image.png

# Test Document Intelligence with custom PDF
cargo run -- test --services document_intelligence \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral \
  --input-file samples/test-document.pdf

# JSON output for automation
cargo run -- test --services all --output json
```

## Service-Specific Examples

### Speech Service

```bash
# Test TTS voices list
cargo run -- test --services speech \
  --scenarios voices_list,tts \
  --api-key YOUR_KEY \
  --region swedencentral

# Test STT with custom audio
cargo run -- test --services speech \
  --scenarios stt_short,stt_rest \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral \
  --input-file samples/test-speech.wav
```

### Translator Service

```bash
# Note: Translator uses global endpoint but needs region header for multi-service keys
cargo run -- test --services translator \
  --api-key YOUR_KEY \
  --region swedencentral
```

### Language Service

```bash
# Test all language scenarios
cargo run -- test --services language \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral

# Test specific scenarios (PII, summarization)
cargo run -- test --services language \
  --scenarios pii_detection,summarization \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral
```

### Vision Service

```bash
# Test with default embedded image
cargo run -- test --services vision \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral

# Test with custom image
cargo run -- test --services vision \
  --input-file samples/test-image.png \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral
```

### Document Intelligence

```bash
# Test with sample PDF
cargo run -- test --services document_intelligence \
  --input-file samples/test-document.pdf \
  --endpoint https://your-resource.cognitiveservices.azure.com \
  --api-key YOUR_KEY \
  --region swedencentral
```

## Generating Custom Test Files

### Audio (WAV)

On macOS, generate WAV files using the `say` command:

```bash
say -o samples/custom-audio.wav --data-format=LEI16@16000 "Your custom text here"
```

Requirements for Azure Speech-to-Text:
- Format: WAV (PCM)
- Sample rate: 16kHz recommended
- Bit depth: 16-bit
- Channels: Mono

### Images

Use any image editor or command-line tools:

```bash
# Using ImageMagick
convert -size 400x200 xc:white \
  -font Helvetica -pointsize 24 \
  -draw "text 20,50 'Test Image'" \
  samples/custom-image.png

# Using sips (macOS)
sips -s format png input.jpg --out samples/custom-image.png
```

### Documents (PDF)

Use any PDF creation tool. The sample `test-document.pdf` contains simple text for layout extraction testing.

## Environment Variables Reference

| Variable | Description |
|----------|-------------|
| `AZURE_AI_API_KEY` | API key for all services |
| `AZURE_AI_ENDPOINT` | Custom subdomain endpoint for all services |
| `AZURE_REGION` | Default region for all services |
| `AZURE_CLOUD` | Cloud environment (global/china) |
| `AZURE_SPEECH_API_KEY` | Speech-specific API key |
| `AZURE_TRANSLATOR_API_KEY` | Translator-specific API key |
| `AZURE_LANGUAGE_API_KEY` | Language-specific API key |
| `AZURE_VISION_API_KEY` | Vision-specific API key |
| `AZURE_DOCUMENT_INTELLIGENCE_API_KEY` | Document Intelligence-specific API key |

## Troubleshooting

### Speech TTS/STT returns 404

Ensure you're using the correct endpoint pattern:
- TTS uses `{region}.tts.speech.microsoft.com`
- STT uses `{region}.stt.speech.microsoft.com`
- Token exchange and Fast Transcription use the custom subdomain

### Translator returns 401 with multi-service key

Ensure the region is set to your resource's actual region (not "global"):
```bash
--region swedencentral  # NOT --region global
```

### Vision caption feature returns 400

The `caption` and `denseCaptions` features are not available in all regions (e.g., swedencentral). Use `tags`, `objects`, `read`, `smartCrops`, or `people` instead.

### Summarization times out

Summarization is an async operation. The tool polls for results but may timeout. If you see "Job submitted (polling timeout, but endpoint responsive)", the endpoint is working correctly.
