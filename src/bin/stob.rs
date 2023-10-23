fn main() {
    let b = b"hello world!";

    let s = String::from_utf8(b.to_vec()).unwrap_or_else(|_| "none".to_string());

    println!("buffer: {:?}, 保存的: {}", b, s);
}
