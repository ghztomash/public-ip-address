fn main() {
    let result = public_ip_address::lookup().unwrap();
    println!("{}", result);
}
