mod first_approach;
mod second_approach;
mod third_approach;

#[tokio::main]
async fn main() {
    let addresses = vec!["example.com:80", "example.com:80", "asdasjskhuessdas.com:80"];
    first_approach::download_files(&addresses);
    second_approach::download_files(&addresses);
    third_approach::download_files(&addresses).await;
}
