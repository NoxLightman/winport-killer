fn main() {
    // 故意分配一点内存，模拟算法题中读取输入
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    let n: i64 = buf.trim().parse().unwrap();
    println!("{}", n * 2);
}
