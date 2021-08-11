use std::pin::Pin;
use std::sync::Arc;
use std::collections::HashMap;

use futures_core::Stream;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

use grpc_streams::grpc_streams_server::{GrpcStreams, GrpcStreamsServer};
use grpc_streams::{Message, Ack};


#[derive(Debug)]
struct GrpcStreamsService { }

impl GrpcStreamsService {
    fn new() -> GrpcStreamsService {
        GrpcStreamsService { }
    }
}

pub mod grpc_streams {
    tonic::include_proto!("grpc_streams");
}

#[tonic::async_trait]
impl GrpcStreams for GrpcStreamsService {

    type PublishStream = Pin<Box<dyn Stream<Item = Result<Ack, Status>> + Send + Sync + 'static>>;

    async fn publish(
        &self,
        _request: Request<tonic::Streaming<Message>>,
    ) -> Result<Response<Self::PublishStream>, Status> {
        let topic = "test-topic".to_string();
        let db = sled::open(format!("data/topics/{}", topic)).unwrap();
        let mut stream = _request.into_inner();
        let output = async_stream::try_stream! {
            while let Some(msg) = stream.message().await? {
                println!("Received {:?}", msg);
                let id = db.generate_id().unwrap();
                db.insert(format!("{}", id).as_bytes(), msg.payload.to_vec()).unwrap();
                yield Ack {
                    key: format!("{}", id).as_bytes().to_vec(), //key.to_vec(),
                    offset: id,
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::PublishStream))
    }
    
}

use std::hash::{Hasher, Hash};

impl Hash for Message {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.key.hash(state);
    }
}

impl Eq for Message {}

use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();
    let service = GrpcStreamsService::new();
    let server = GrpcStreamsServer::new(service);
    Server::builder()
        .add_service(server)
        .serve(addr)
        .await?;

    Ok(())
}
