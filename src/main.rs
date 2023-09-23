use std::io;
use std::io::Error;
use http_body_util::Empty;
use hyper::{Request, Response, Uri};
use hyper::body::{Bytes, Incoming};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use http_body_util::BodyExt;
use hyper::client::conn::http1::SendRequest;
use tokio::io::{stdout, AsyncWriteExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "http://httpbin.org/ip".parse::<Uri>()?;

    // Setup our HTTP client...
    let mut sender = setup_http_client(&url).await?;

    // Create a request...
    let req = create_request(&url, url.authority().unwrap()).await?;

    // Await the response...
    let mut res = sender.send_request(req).await?;

    println!("Response status: {}", res.status());

    // Read the response body...
    read_response(&mut res).await?;

    Ok(())
}


async fn setup_http_client(url: &Uri) -> Result<SendRequest<Empty<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    // Get the host and the port
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{}:{}", host, port);

    // Open a TCP connection to the remote host
    let stream = TcpStream::connect(address).await?;

    // Use an adapter to access something implementing `tokio::io` traits as if they implement
    // `hyper::rt` IO traits.
    let io = TokioIo::new(stream);

    // Perform a TCP handshake
    let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    Ok(sender)
}

async fn read_response(res: &mut Response<Incoming>) -> Result<(), Error> {
    while let Some(next) = res.frame().await {
        let frame = match next {
            Ok(frame) => frame,
            Err(err) => return Err(Error::new(io::ErrorKind::Other, err)),
        };

        if let Some(chunk) = frame.data_ref() {
            stdout().write_all(&chunk).await?;
        }
    }

    Ok(())
}

async fn create_request(url: &Uri, authority: &hyper::http::uri::Authority) -> Result<Request<Empty<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;
    Ok(req)
}