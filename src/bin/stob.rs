#[tokio::main]
async fn main() {
    let b = "hello world 滚耳朵!".as_bytes();

    let s = String::from_utf8(b.to_vec()).unwrap_or_else(|_| "none".to_string());

    println!("buffer: {:?}, 保存的: {}", b, s);
}

// include!(concat!("../..", "/hello.rs"));
