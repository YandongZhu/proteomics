# LoadData node image.
#
# Build using the repo root as the build context (so proto/workflow.proto is available):
#   docker build -f docker/load_data/Dockerfile -t proteomic/load-data:latest .
FROM python:3.11-slim

WORKDIR /app

# grpcio-tools bundles protoc, used at image build time to generate
# workflow_pb2.py, so proto/workflow.proto remains the single source of truth.
RUN pip install --no-cache-dir grpcio-tools==1.* protobuf==5.*

COPY proto/workflow.proto /app/proto/workflow.proto
RUN python -m grpc_tools.protoc \
        -I /app/proto \
        --python_out=/app \
        /app/proto/workflow.proto

COPY docker/load_data/load_data.py /app/load_data.py

ENTRYPOINT ["python", "/app/load_data.py"]
