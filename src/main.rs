mod structs;
use std::net::TcpListener;
use std::io::Read;
use local_ip_address::local_ip;
use serde_json::Value;
use barcode_scanner::BarcodeScanner;

fn read_barcode() -> Result<(), barcode_scanner::Error>
{
        let mut scanner = BarcodeScanner::open("/dev/input/by-id/usb-ADESSO_NuScan_1600U-event-kbd")?;
        loop {
                scanner.read()?
        }
}

fn get_config() -> serde_json::Value {
    let my_local_ip_address: String = "127.0.0.1".to_string();
    match local_ip()
    {
        Ok(t) => {let my_local_ip_address = t.to_string(); println!("Local address: {}", my_local_ip_address)},
        Err(_e) => {println!("Couldn't get ip address correctly, resorting to loopback");}
    }
    
    let mut rx: String = "".to_string();

    match TcpListener::bind("127.0.0.1".to_string() + ":48753")
    {
        Ok(t) => {
            println!("Address binded, waiting for connection");
            for stream in t.incoming()
            {
                match stream
                {
                    Ok(mut stream) => {
                        match stream.read_to_string( &mut rx) // read_to_string could be used with a json parser or changed to read which gives a byte array
                        {
                            Ok(t) => {println!("Extracted JSON of size {} bytes", t)},
                            Err(e) => println!("Error occured at reading packet to buffer\n{}", e)
                        }
                        break;
                    },
                    Err(e) => println!("Error occured at match incoming\n{}", e)
                }
            }
        },
        Err(e) => println!("Error occured at TCP Bind\n{}", e)
    }

    match serde_json::from_str(&rx)
    {
        Ok(t) => {return t},
        Err(e) => {println!("Error parsing JSON: {}", e); serde_json::from_str("error").unwrap()} // Semi-hack fix but it SHOULDN'T ever error.  Mint
    }
}

fn main() {
    // Get config from network
    let rx_config: Value = serde_json::from_str(r#"
    {"route": [
        {"destination_id": 201, "x": 0, "y": 0}, 
        {"destination_id": 202, "x": 10, "y": 0},
        {"destination_id": 203, "x": 0, "y": 10},
        {"destination_id": 204, "x": 10, "y": 10},
        {"destination_id": 205, "x": 20, "y": 10}],
    "box_loc": [
        {"tracking_number": 101, "destination": 201},
        {"tracking_number": 102, "destination": 201},
        {"tracking_number": 103, "destination": 201},
        {"tracking_number": 104, "destination": 202},
        {"tracking_number": 105, "destination": 203},
        {"tracking_number": 106, "destination": 204},
        {"tracking_number": 107, "destination": 204},
        {"tracking_number": 108, "destination": 204},
        {"tracking_number": 109, "destination": 205}]}"#).unwrap();

    let rx_config: Value = get_config();

    // Create Package Vector
    let mut packages: Vec<structs::BoxStruct> = vec![];
    let mut route: Vec<structs::RouteStruct> = vec![];

    {
        // Create temp route vector for sanitizing purposes
        let mut temp_route_id: Vec<i64> = vec![];
        let mut temp_route_x: Vec<i64> = vec![];
        let mut temp_route_y: Vec<i64> = vec![];

        for i in 0..5
        {
            match rx_config["route"][i]["destination_id"].as_i64()
            {
                None => {println!("parser said fuck")},
                Some(t) => {temp_route_id.push(t)}
            }
        }

        for i in 0..5
        {
            match rx_config["route"][i]["x"].as_i64()
            {
                None => {println!("parser said fuck")},
                Some(t) => {temp_route_x.push(t)}
            }
        }

        for i in 0..5
        {
            match rx_config["route"][i]["y"].as_i64()
            {
                None => {println!("parser said fuck")},
                Some(t) => {temp_route_y.push(t)}
            }
        }

        for i in 0..5
        {
            route.push(structs::RouteStruct{
                destination_id: temp_route_id[i],
                x_pos: temp_route_x[i],
                y_pos: temp_route_y[i]
            });
        }

        // Create temp package vector for sanitizing purposes
        let mut temp_pack: Vec<i64> = vec![];
        let mut temp_dest: Vec<i64> = vec![];

        for i in 0..9
        {
            match rx_config["box_loc"][i]["tracking_number"].as_i64()
            {
                None => {println!("parser said fuck")},
                Some(t) => {temp_pack.push(t)}
            }
        }

        for i in 0..9
        {
            match rx_config["box_loc"][i]["destination"].as_i64()
            {
                None => {println!("parser said fuck 2"); temp_dest.push(404)},
                Some(t) => {temp_dest.push(t)}
            }
        }

        // Push network config to package vector
        for i in 0..9
        {
            packages.push(structs::BoxStruct{
                tracking_number: temp_pack[i],
                destination: temp_dest[i],
                x_pos: i as u32 * 2,
                y_pos: i as u32 * 2
            });
        }
    }

    // Prints package tracking numbers to verify loading into packages vector
    for i in 0 .. packages.len()
    {
        println!("TN: {} => {}", packages[i].tracking_number, packages[i].destination);
    }
    
    // Prints route information for debugging
    for i in 0 .. route.len()
    {
        println!("{}", route[i].destination_id);
    }

    // Create barcode scanner
    match BarcodeScanner::open("/dev/input/by-id/usb-ADESSO_NuScan_1600U-event-kbd")
    {
        Ok(mut t) => {
            match t.read()
            {
                Ok(t) => {println!("{}", t);},
                Err(e) => {println!("Couldn't read: {}", e);}
            }},
        Err(e) => {println!("Couldn't find device: {}", e);}
    }

}

/*
todo
1. Make fn route_sort() that takes in the BoxStructs on the shelves and assigns them places on the shelves
2. Make async fn move_x and async fn move_y to simulate motor control
 
1. Take in full route and boxmap from ethernet
2. Sort route from set map
3. Post location on Google map embedded in website
4. Time to move from one box location to another (appox 2"/s)
 
JSON:
{
    "route": [{"destination_id": 20x,"x": x, "y": y}, {"destination_id": 20x,"x": x, "y": y}, ...],
    "box_loc": [{"box_index": x1, "route_stop": route[x1]}, {"box_index": x1, "route_stop": route[x1]}],
}


- Initialize motors and encoders
- Set motors to safe spot
- Check for door switch and disable until off
- Ask network for box map and route if updated version is not stored
- Get route and box dropoff locations from the network JSON
- Scan shelf for physical box locations
- 

*/