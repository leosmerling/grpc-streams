use std::pin::Pin;
// use std::sync::Arc;
// use std::collections::HashMap;

use futures_core::Stream;
// use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
// use tokio_stream::wrappers::ReceiverStream;

use grpc_streams::grpc_streams_server::{GrpcStreams, GrpcStreamsServer};
use grpc_streams::{Consumer, Message, Ack};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref DB: sled::Db = sled::open("data/topics/test-topic").unwrap();
}

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
    type ConsumeStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send + Sync + 'static>>;

    async fn publish(
        &self,
        _request: Request<tonic::Streaming<Message>>,
    ) -> Result<Response<Self::PublishStream>, Status> {
        let mut stream = _request.into_inner();
        let output = async_stream::try_stream! {
            while let Some(msg) = stream.message().await? {
                println!("Received {:?}", msg);
                let id = DB.generate_id().unwrap();
                DB.insert(format!("{:12}", id).as_bytes(), msg.payload.to_vec()).unwrap();
                yield Ack {
                    key: format!("{:12}", id).as_bytes().to_vec(), //key.to_vec(),
                    offset: id,
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::PublishStream))
    }

    async fn consume(
        &self,
        _request: Request<Consumer>,
    ) -> Result<Response<Self::ConsumeStream>, Status> {

        let output = async_stream::try_stream! {
            let mut offset = format!("{:12}", 0).as_bytes().to_vec();
            while let Ok(Some((key, payload))) = DB.get_gt(&offset) {
                println!("Consuming {:?}: {:?}", key, payload);
                offset = key.to_vec();
                yield Message {
                    topic: "test-topic".to_string(),
                    key: key.to_vec(),
                    payload: payload.to_vec()
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::ConsumeStream))
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
