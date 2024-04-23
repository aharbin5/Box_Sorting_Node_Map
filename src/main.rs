use std::error::Error;
use std::thread;
use std::time::Duration;
use rppal::gpio::Gpio;
use rppal::pwm::*;
use barcode_scanner::BarcodeScanner;
mod extra;

use linux_embedded_hal::I2cdev;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use xca9548a::{Xca9548a, SlaveAddr};

const HORIZONTAL_DIRECTION: u8 = 14;
const VERTICAL_DIRECTION: u8 = 15;
const SPREADER_DIRECTION: u8 = 23;
const FORKLIFT_DIRECTION: u8 = 24;

use std::sync::mpsc;

use as5600::As5600;

fn main() {
    let (main_tx, main_rx) = mpsc::channel();
    let (thread_tx, thread_rx) = mpsc::channel();
    let (scanner_tx, scanner_rx) = mpsc::channel::<String>();
    /* 
        #### Main mpsc Status Guide ####
        Bit 0 - Code
        Bit 1 - Data

        0,0 - disable and kill the thread
        0,1 - disable pwm
        0,2 - enable pwm then wait for target
        1,? - target message

        #### Thread mpsc Status Guide ####
        Bit 0 - Code
        Bit 1 - Unused

        0,0 - Success
        1,0 - Error
    */
    let gpio = Gpio::new().unwrap();
    let mut horizontal_gpio = gpio.get(HORIZONTAL_DIRECTION).unwrap().into_output();
    let mut forklift_gpio = gpio.get(FORKLIFT_DIRECTION).unwrap().into_output();

    let splitter = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let i2c_switch = Xca9548a::new(splitter, address);
    let parts = i2c_switch.split();
    //let horizontal_i2c = parts.i2c0;
    let forklift_i2c = parts.i2c1;

    //let i2c = I2cdev::new("/dev/i2c-1").unwrap(); // set encoder on default bus
    let mut horizontal_encoder = As5600::new(I2cdev::new("/dev/i2c-0").unwrap());
    println!("Horizontal: {:?}", horizontal_encoder.config().unwrap());
    println!("Encoder read: {:?}", horizontal_encoder.angle().unwrap());

    let mut forklift_encoder = As5600::new(forklift_i2c);
    println!("Forklift: {:?}", forklift_encoder.config().unwrap());
    println!("Encoder read: {:?}", forklift_encoder.angle().unwrap());

    let mut total_rotations: i32 = 0;

    let mut current_quadrant = 1;
    let mut previous_quadrant = 1;

    let _ = main_tx.send([0,1]).unwrap();

    let mut current_position: i32 = 0;

    // PWM Thread Init
    let pwm_thread = thread::spawn(move ||
    {
	let mut target_position: i32;
        let pwm = rppal::pwm::Pwm::with_frequency(Channel::Pwm0, 3200 as f64, 0.25, Polarity::Normal, false).unwrap();
	println!("thread spawned and pwm set false");

        let initial_angle: i32 = horizontal_encoder.angle().unwrap() as i32;

        loop {
		let status = main_rx.recv().unwrap();
		if status[0] == 0 && status[1] == 1 {
			let _ = pwm.disable();
			println!("disabled for now");
		} 
		else if status[0] == 0 && status[1] == 2 {
			let _ = pwm.enable();
			let target_position = main_rx.recv().unwrap()[1] - initial_angle; // Rx target position [1,?]
			println!("enabled with target: {}", target_position);
			loop {
				let raw_angle = horizontal_encoder.angle().unwrap() as i32;
			        //let polar_angle: f32 = ((raw_angle as f32 / 4096.0) * 360.0) as f32; // For display purposes ONLY
			        previous_quadrant = current_quadrant;
			        current_quadrant = match raw_angle {
			                0 ..= 1024 => {1},
			                1025 ..= 2048 => {4},
			                2049 ..= 3072 => {3},
			                3073 ..= 4096 => {2},
			                _ => {println!("could not find quadrant"); -1} // Failure code
        			};
			        if previous_quadrant == 1 && current_quadrant == 2 {total_rotations -= 1;} 
                                else if previous_quadrant == 2 && current_quadrant == 1 {total_rotations += 1;} 
                                else if previous_quadrant == -1 || current_quadrant == -1 {
                                        thread_tx.send([1,0]).unwrap();
                                        let _ = pwm.disable();
                                        break;
                                }
				current_position = (total_rotations * 4096) + raw_angle as i32 - initial_angle;
			        //println!("{:?}\t|\t{:?}\tTotal Angle: {}", polar_angle, current_quadrant, current_position); // For  debugging, comment out in real run
				thread::sleep(Duration::from_millis(10));
				if current_position < target_position + 50 && current_position > target_position - 50 {
					println!("hit target: current {} ~ target {}", current_position, target_position);
					let _ = pwm.disable();
                                        thread_tx.send([0,0]).unwrap(); // Success code
					break;
			        } else if current_position < target_position {
					horizontal_gpio.set_high();
					if !pwm.is_enabled().unwrap() {
						let _ = pwm.enable();
					}
				} else if current_position > target_position {
					horizontal_gpio.set_low();
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
	}
    });

    // Barcode Scanner Thread Init
    let scanner_thread = thread::spawn(move ||
    {
	let mut scanned_counter = 0;
	match BarcodeScanner::open("/dev/input/by-id/usb-ADESSO_NuScan_1600U-event-kbd")
        {
                Ok(mut t) => {
			while scanned_counter < 10 {
				match t.read() {
                       		Ok(t2) => {
					match t2.parse::<i32>().unwrap() {
						999..=8999 => {let _ = scanner_tx.send(t2).unwrap();},
						_ => {println!("who knows what was scanned");}, // change later to be more useful
					}
					scanned_counter += 1;},
                        	Err(e) => {println!("couldn't read"); scanned_counter = 100;}
                        	}
			}
                },
                Err(e) => {println!("could not find device");}
        }
    });

    println!("thread is waiting 10s, scan fast");
    thread::sleep(Duration::from_secs(10));

    while let Ok(i) = scanner_rx.try_recv() {
	println!("{}", i);
    }

    println!("we finished the list boys");

    scanner_thread.join().unwrap();
/*
    // Move robot to -8192 and wait 5s
    extra::move_horizontal(&main_tx, -8192);
    thread::sleep(Duration::from_secs(5)); // Change this to read the thread_rx to know when to move
    
    // Initialize forklift_pwm and load a box in the current position
    let mut forklift_pwm = rppal::pwm::Pwm::with_frequency(Channel::Pwm1, 3200 as f64, 0.25, Polarity::Normal, false).unwrap();
    extra::load_box(&mut forklift_pwm, &mut forklift_gpio);

    // Move robot to 0 and wait 1s
    extra::move_horizontal(&main_tx, 0);
    thread::sleep(Duration::from_secs(1));

    // Stupid vibrate function
    forklift_gpio.set_low();
    let _ = forklift_pwm.enable();
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
    let _ = forklift_pwm.disable();

    forklift_gpio.set_high();
    let _ = forklift_pwm.enable();
    thread::sleep(Duration::from_secs(1));
    let _ = forklift_pwm.disable();
*/
    // Wrap things up by killing the thread
    println!("send kill");
    let _ = main_tx.send([0, 0]);
    pwm_thread.join().unwrap();
    println!("Killed, Quitting"); 
}
