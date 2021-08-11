pub mod grpc_streams {
    tonic::include_proto!("grpc_streams");
}

use grpc_streams::grpc_streams_client::GrpcStreamsClient;
use grpc_streams::{Message, Consumer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrpcStreamsClient::connect("http://[::1]:10000").await?;
    println!("Connected!");
    consume_messages(&mut client).await?;
    Ok(())
}

use std::str;
use std::time::Duration;
use tokio::time;
use tonic::Request;
use tonic::transport::Channel;

async fn consume_messages(client: &mut GrpcStreamsClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {

    let consumer = Consumer {
        topic: "test-topic".to_string(),
        group: "test-group".to_string()
    };
    let response = client.consume(Request::new(consumer)).await?;
    let mut inbound = response.into_inner();
    println!("Got response!");
    while let Some(msg) = inbound.message().await? {
        println!("Consumed {:?} {:?}", msg, str::from_utf8(&msg.key));
    }

    Ok(())

}
