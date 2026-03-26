use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("gRPC transport: {0}")]
    GrpcTransport(String),

    #[error("QuestDB: {0}")]
    QuestDb(String),

    #[error("postgres: {0}")]
    Postgres(#[from] tokio_postgres::Error),

    #[error("query: {0}")]
    Query(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("decode: {0}")]
    Decode(String),

    #[error("channel closed")]
    ChannelClosed,
}

impl From<Error> for tonic::Status {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidRequest(msg) => tonic::Status::invalid_argument(msg),
            Error::Query(msg) => tonic::Status::not_found(msg),
            Error::Postgres(e) => tonic::Status::internal(e.to_string()),
            Error::Grpc(s) => s,
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
