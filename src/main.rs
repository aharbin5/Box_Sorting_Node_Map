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

const HORIZONTAL_PWM: u8 = 12;
//const VERTICAL_PWM: u8 = 13; // This is initialized in extra.rs, no need to do it here
const FORKLIFT_PWM_PIN: u8 = 5;
const SPREADER_PWM_PIN: u8 = 6;

const HORIZONTAL_DIRECTION: u8 = 14;
//const VERTICAL_DIRECTION: u8 = 15; // This is initialized in extra.rs, no need to do it here
const SPREADER_DIRECTION: u8 = 23;
const FORKLIFT_DIRECTION: u8 = 24;

use std::sync::mpsc;

use as5600::As5600;

fn main() {
    let (mut main_tx, main_rx) = mpsc::channel(); // Used to tell the horizontal thread to do stuff
    let (thread_tx, thread_rx) = mpsc::channel(); // Used to recieve stuff from the horizontal thread
    let (scanner_tx, scanner_rx) = mpsc::channel(); // Used for the scanner thread and the horiztonal thread to talk
    /* 
        #### Main mpsc Status Guide ####
        Bit 0 - Code
        Bit 1 - Data

        0,0 - disable and kill the thread
        0,1 - disable pwm
        0,2 - enable pwm then wait for target
        0,3 - Send current encoder position
        1,{target_position} - target message

        #### Thread mpsc Status Guide ####
        Bit 0 - Code
        Bit 1 - Unused

        0,0 - Success
        1,0 - Error
        {{package_id}, {position}} - Box location
    */
    let gpio = Gpio::new().unwrap();
    
    let mut horizontal_pwm = gpio.get(HORIZONTAL_PWM).unwrap().into_output();
    //let mut vertical_pwn = gpio.get(VERTICAL_PWM).unwrap().into_output(); // This is initialized in extra.rs, no need to do it here
    let mut forklift_pwm = gpio.get(FORKLIFT_PWM_PIN).unwrap().into_output();
    let mut spreader_pwm = gpio.get(SPREADER_PWM_PIN).unwrap().into_output();

    let mut horizontal_gpio = gpio.get(HORIZONTAL_DIRECTION).unwrap().into_output();
    //let mut vertical_gpio = gpio.get(VERTICAL_DIRECTION).unwrap().into_output(); // This is initialized in extra.rs, no need to do it here
    let mut forklift_gpio = gpio.get(FORKLIFT_DIRECTION).unwrap().into_output();
    let mut spreader_gpio = gpio.get(SPREADER_DIRECTION).unwrap().into_output();

    println!("All GPIO & PWM pins initialized");

    let splitter = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let i2c_switch = Xca9548a::new(splitter, address);
    let parts = i2c_switch.split();
    
    let vertical_i2c = parts.i2c0;
    let forklift_i2c = parts.i2c1;
    let spreaders_i2c = parts.i2c2;

    println!("Multiplexer initialized");

    //let i2c = I2cdev::new("/dev/i2c-1").unwrap(); // set encoder on default bus
    let mut horizontal_encoder = As5600::new(I2cdev::new("/dev/i2c-0").unwrap());
    //println!("Horizontal: {:?}", horizontal_encoder.config().unwrap());

    let mut vertical_encoder = As5600::new(vertical_i2c);
    //println!("Vertical: {:?}", vertical_encoder.config().unwrap());

    let mut forklift_encoder = As5600::new(forklift_i2c);
    //println!("Forklift: {:?}", forklift_encoder.config().unwrap());

    let mut spreaders_i2c = As5600::new(spreaders_i2c);
    //println!("Spreaders: {:?}", forklift_encoder.config().unwrap());

    println!("Horizontal encoder initialized");

    let _ = main_tx.send([0,1]).unwrap();

    //let mut current_position: i32 = 0;

    // PWM Thread Init
    let pwm_thread = thread::spawn(move ||
    {
        let mut total_rotations: i32 = 0;
        let mut current_quadrant = 1;
        let mut previous_quadrant = 1;
        let mut target_position: i32;
	let mut current_position: i32 = 0;
        // horizontal_pwm.set_pwm_frequency(freq: f64, duty: f64);
        println!("Horizontal thread spawned and pwm set false");

            let initial_angle: i32 = horizontal_encoder.angle().unwrap() as i32;

            loop {
            let status = main_rx.recv().unwrap();
            if status[0] == 0 && status[1] == 1 {
                let _ = horizontal_pwm.clear_pwm();
                println!("disabled for now");
            } 
            else if status[0] == 0 && status[1] == 2 {
                let _ = horizontal_pwm.set_pwm_frequency(3200 as f64, 0.5 as f64);
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
                       let _ = horizontal_pwm.clear_pwm();
                        break;
                    }
                    current_position = (total_rotations * 4096) + raw_angle as i32 - initial_angle;
                    //println!("{:?}\tTotal Angle: {}", current_quadrant, current_position); // For  debugging, comment out in real run
                    thread::sleep(Duration::from_millis(10));
		    
		    match scanner_rx.try_recv() {
			Ok(t) => {println!("{} at {}", t, current_position); thread_tx.send([0,0]).unwrap();}, // Change this to send back to main later to be put in the actual boxmap
			Err(_e) => {}
		    }
                    if current_position < target_position + 50 && current_position > target_position - 50 {
                        println!("hit target: current {} ~ target {}", current_position, target_position);
                        let _ = horizontal_pwm.clear_pwm();
                        thread_tx.send([0,0]).unwrap(); // Success code
                        break;
                    } else if current_position < target_position {
                        horizontal_gpio.set_high();
                        let _ = horizontal_pwm.set_pwm_frequency(3200 as f64, 0.5 as f64);
                    } else if current_position > target_position {
                        horizontal_gpio.set_low();
                        let _ = horizontal_pwm.set_pwm_frequency(3200 as f64, 0.5 as f64);
                    }
                }
            }
            else if status[0] == 0 && status[1] == 0 {
                let _ = horizontal_pwm.clear_pwm();
                println!("pwm disabled and killed");
                break;
            }
        }
    });

    // Barcode Scanner Thread Init
    let scanner_thread = thread::spawn(move ||
    {
	let mut scanned_counter = 0;
    let string_thing: String;
	match BarcodeScanner::open("/dev/input/by-id/usb-ADESSO_NuScan_1600U-event-kbd")
        {
                Ok(mut t) => {
			loop {
				match t.read() {
                    Ok(t2) => {
                    let string_thing = t2.split_at(t2.len()-1).0.to_string();
					match string_thing.parse::<i32>().unwrap() {
						999..=8999 => {let _ = scanner_tx.send(string_thing.parse::<i32>().unwrap());},
						_ => {println!("Something weird was scanned, breaking"); break;},
					}
					scanned_counter += 1;},
                        	Err(_e) => {println!("Barcode read did not return code and errored?"); break;}
                        	}
			}
                },
                Err(_e) => {println!("Could not find scanner");}
        }
    });

    // Package list to add to while scanning shelves
    let mut packages: Vec<extra::BoxStruct> = vec![];

    let mut current_shelf: i32 = 0;
    current_shelf = extra::goto_shelf(1, &vertical_encoder);

    thread::sleep(Duration::from_secs(5));

    current_shelf = extra::goto_shelf(0, &vertical_encoder);

    extra::move_horizontal(&mut main_tx, -16384);
    loop {
        let mut current_packet = thread_rx.recv().unwrap();
        match current_packet[0] {
            0 => {break;}
            1 => {
                println!("Error code 1 from Horizontal, waiting 5s and retrying");
                thread::sleep(Duration::from_secs(5));
                extra::move_horizontal(&main_tx, -8192);}, // This is potentially unsafe if the initial_position gets reset
            999..=8999 => {packages.push(extra::BoxStruct{x_pos: current_packet[1], y_pos: current_shelf, id: current_packet[0]});},
            _ => {println!("Horizontal thread sent something weird back?");},
        }
    }

    // Initialize forklift_pwm and load a box in the current position
    extra::load_box(&mut forklift_pwm, &mut forklift_gpio, &mut forklift_encoder);

    extra::move_horizontal(&main_tx, 0);

    // Wrap things up by killing the thread
    println!("send kill");
    let _ = main_tx.send([0, 0]);
    pwm_thread.join().unwrap();
    scanner_thread.join().unwrap();
    println!("Killed, Quitting"); 
}
