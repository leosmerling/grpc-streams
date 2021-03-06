pub mod grpc_streams {
    tonic::include_proto!("grpc_streams");
}

use grpc_streams::grpc_streams_client::GrpcStreamsClient;
use grpc_streams::{Message, Ack};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrpcStreamsClient::connect("http://[::1]:10000").await?;
    publish_messages(&mut client).await?;
    Ok(())
}


use std::time::Duration;
use tokio::time;
use tonic::Request;
use tonic::transport::Channel;

async fn publish_messages(client: &mut GrpcStreamsClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {

    let outbound = async_stream::stream! {
        let mut interval = time::interval(Duration::from_secs(1));

        while let _ = interval.tick().await {
            let msg = Message {
                topic: "test-topic".to_string(),
                key: b"123".to_vec(),
                payload: b"sample-message".to_vec(),
            };
            yield msg;
        }
    };

    let response = client.publish(Request::new(outbound)).await?;
    let mut inbound = response.into_inner();

    while let Some(ack) = inbound.message().await? {
        println!("Ack = {:?}", ack);
    }

    Ok(())

}
