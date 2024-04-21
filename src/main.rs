use std::error::Error;
use std::thread;
use std::time::Duration;
use rppal::gpio::Gpio;
use rppal::pwm::*;

use linux_embedded_hal::I2cdev;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use xca9548a::{Xca9548a, SlaveAddr};

const horizontal_direction: u8 = 14;
const forklift_direction: u8 = 4;

use std::sync::mpsc;

use as5600::As5600;

fn main() {
    let (main_tx, main_rx) = mpsc::channel();
    let (thread_tx, thread_rx) = mpsc::channel();
    let gpio = Gpio::new().unwrap();
    let mut direction_pin = gpio.get(horizontal_direction).unwrap().into_output();
    let mut forklift_gpio = gpio.get(forklift_direction).unwrap().into_output();

    let splitter = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let i2c_switch = Xca9548a::new(splitter, address);
    let parts = i2c_switch.split();
    //let horizontal_i2c = parts.i2c0;
    let forklift_i2c = parts.i2c1;

    //let i2c = I2cdev::new("/dev/i2c-1").unwrap(); // set encoder on default bus
    let mut horizontal_encoder = As5600::new(I2cdev::new("/dev/i2c-0").unwrap());
    println!("Horizontal: {:?}", horizontal_encoder.config().unwrap());

    let mut forklift_encoder = As5600::new(forklift_i2c);
    println!("Forklift: {:?}", forklift_encoder.config().unwrap());

    let mut total_rotations: i32 = 0;

    let mut current_quadrant = 1;
    let mut previous_quadrant = 1;

    let _ = main_tx.send([0,1]).unwrap();

    let mut current_position: i32 = 0;

    let pwm_thread = thread::spawn(move ||
    {
	let mut target_position: i32;
        let pwm = rppal::pwm::Pwm::with_frequency(Channel::Pwm0, 3200 as f64, 0.25, Polarity::Normal, false).unwrap();
	println!("thread spawned and pwm set false");

        let initial_angle: i32 = horizontal_encoder.angle().unwrap() as i32;

        loop {
		println!("top of loop");
		let status = main_rx.recv().unwrap();
		if status[0] == 0 && status[1] == 1 {
			let _ = pwm.disable();
			println!("disabled for now");
		} 
		else if status[0] == 0 && status[1] == 2 {
			let _ = pwm.enable();
			let target_position = main_rx.recv().unwrap()[1] - initial_angle; // enter posiiton
			println!("enabled with target: {}", target_position);
			loop {
				let raw_angle = horizontal_encoder.angle().unwrap() as i32;
			        let polar_angle: f32 = ((raw_angle as f32 / 4096.0) * 360.0) as f32; // For display purposes ONLY
			        previous_quadrant = current_quadrant;
			        current_quadrant = match raw_angle {
			                0 ..= 1024 => {1},
			                1025 ..= 2048 => {4},
			                2049 ..= 3072 => {3},
			                3073 ..= 4096 => {2},
			                _ => {println!("could not find quadrant"); -1}
        			};
			        if previous_quadrant == 1 && current_quadrant == 2 {total_rotations -= 1;}
			        else if previous_quadrant == 2 && current_quadrant == 1 {total_rotations += 1;}
				current_position = (total_rotations * 4096) + raw_angle as i32 - initial_angle;
			        //println!("{:?}\t|\t{:?}\tTotal Angle: {}", polar_angle, current_quadrant, current_position);
				thread::sleep(Duration::from_millis(10));
				if current_position < target_position + 50 && current_position > target_position - 50 {
					println!("hit target: current {} ~ target {}", current_position, target_position);
					let _ = pwm.disable();
					break;
			        } else if current_position < target_position {
					direction_pin.set_high();
					if !pwm.is_enabled().unwrap() {
						let _ = pwm.enable();
					}
				} else if current_position > target_position {
					direction_pin.set_low();
					if !pwm.is_enabled().unwrap() {
						let _ = pwm.enable();
					}
				}
			}
		}
		else if status[0] == 0 && status[1] == 0 {
			let _ = pwm.disable();
			println!("thread disabled and killed");
			break;
		}
		else if status[1] == 1 {
			let _ = thread_tx.send([1,0]).unwrap();
		}
	}
    });
    //thread::sleep(Duration::from_secs(1));	// TODO
    let _ = main_tx.send([0, 2]);
    println!("sent enable");
    thread::sleep(Duration::from_millis(10));
    let _ = main_tx.send([4, -8192]);
    println!("sent target position and waiting 5s");
    thread::sleep(Duration::from_secs(5));
    
    let pwm2 = rppal::pwm::Pwm::with_frequency(Channel::Pwm1, 3200 as f64, 0.25, Polarity::Normal, false).unwrap();
    forklift_gpio.set_low();
    let _ = pwm2.enable();
    thread::sleep(Duration::from_secs(2));
    forklift_gpio.set_high();
    thread::sleep(Duration::from_secs(2));
    let _ = pwm2.disable();

    let _ = main_tx.send([0, 2]);
    thread::sleep(Duration::from_millis(10));
    let _ = main_tx.send([4, 0]);
    thread::sleep(Duration::from_secs(5));

    forklift_gpio.set_low();
    let _ = pwm2.enable();
    thread::sleep(Duration::from_secs(1));
    let mut counter = 0;
    loop {
	if counter > 75 {break;}
        forklift_gpio.set_high();
        thread::sleep(Duration::from_millis(20));
        forklift_gpio.set_low();
        thread::sleep(Duration::from_millis(20));
	counter += 1;
    }
    let _ = pwm2.disable();

    forklift_gpio.set_high();
    let _ = pwm2.enable();
    thread::sleep(Duration::from_secs(1));
    let _ = pwm2.disable();


    /*println!("send target2 position and wait 5s");
    let _ = main_tx.send([0, 2]);
    thread::sleep(Duration::from_millis(10));
    let _ = main_tx.send([4, 0]);
    thread::sleep(Duration::from_secs(5));*/
    

    println!("send kill");
    let _ = main_tx.send([0, 0]);
    //let _ = tx.send(true).unwrap();		// TODO
    //thread::sleep(Duration::from_secs(10));	// TODO
    //let _ = tx.send(false);			// TODO
    //pwm_thread.join().unwrap();		// TODO
    pwm_thread.join().unwrap();
    println!("joined and finish"); 
}