use clap::Parser;
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

#[derive(Parser)]
struct Cli {
    address: String,
    #[arg(long, default_value_t=String::from("http"))]
    protocol: String,
    #[arg(short, long, default_value_t=String::from("5000"))]
    port: String,
}

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let endpoint = format!(
        "{protocol}://{addr}:{port}",
        protocol = args.protocol,
        addr = args.address,
        port = args.port
    );
    let mut client = GreeterClient::connect(endpoint).await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    println!("REQUEST={:?}", request);
    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
