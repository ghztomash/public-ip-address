fn main() {
    let result = public_ip_address::get_response().unwrap();
    println!("{:#?}", result);
}
