ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}python:3.11-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir \
    nats-py==2.9.0 \
    aiohttp==3.9.1

# Copy test harness
COPY test_harness.py /app/

# Run test
CMD ["python", "-u", "/app/test_harness.py"]
