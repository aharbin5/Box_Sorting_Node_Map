#[interrupt]
fn something_is_wrong() {
    println!("Door interrupt called");
    panic!();
}