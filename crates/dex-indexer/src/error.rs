use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("gRPC transport: {0}")]
    GrpcTransport(String),

    #[error("QuestDB: {0}")]
    QuestDb(String),

    #[error("decode: {0}")]
    Decode(String),

    #[error("channel closed")]
    ChannelClosed,
}
