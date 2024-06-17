use url::Url;
use std::env::var;

fn main() {
    let url = var("PUBLIC_URL")
        .unwrap_or("http://localhost:3000".to_string());

    let url = Url::parse(&url)
        .expect("Invalid PUBLIC_URL");

    println!("cargo:rustc-env=BUILD_PUBLIC_URL={}", url.origin().ascii_serialization());
}
