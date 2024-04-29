
// This is supposed to be an interrupt callback function but it doesn't want me to label it as
// an #interrupt so I'm not going to.  This'll be fine
pub fn something_is_wrong(_level: rppal::gpio::Level) {
    println!("Door interrupt called");
    panic!();
}
