FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY api_service.py .

EXPOSE 8000

CMD ["python", "api_service.py"]
