mod structs;
use std::net::TcpListener;
use std::io::Read;
use local_ip_address::local_ip;
use serde_json::Value;

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
                            Ok(_t) => {println!("{}", rx)},
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
        Err(e) => {println!("{}", e); serde_json::from_str("fuck").unwrap()} // Semi-hack fix but it SHOULDN'T ever error.  Mint
    }
}

fn main() {
    // Real package Vector
    let mut packages: Vec<structs::BoxStruct> = vec![];

    {   //////////////////////////////// Setup package list ////////////////////////////////
        // xpos 0 .. 1024, ypos 0 .. 1024, tracking number 100 .. 110, destination 1 .. 9
        let temp_packages = [
            [0, 10, 100, 5],
            [100, 50, 101, 1],
            [200, 120, 102, 2],
            [500, 320, 103, 7],
            [900, 390, 104, 10],
            [1024, 640, 105, 6],
            [152, 440, 106, 8],
            [30, 710, 107, 1],
            [770, 250, 108, 8],
            [690, 909, 109, 4]
        ];

        for i in 0..10
        {
            packages.push(structs::BoxStruct{
                x_pos: temp_packages[i][0],
                y_pos: temp_packages[i][1],
                tracking_number: temp_packages[i][2],
                destination: temp_packages[i][3]
            });
        }
    }

    // Real connections with weights Vectors
    // Nodes are 1 .. 10 inclusive
    let mut connections:Vec<structs::ConnectionStruct> = vec![];

    {   //////////////////////// Populate nodemap with connections //////////////////////
        
        let temp_connections = 
        [
            [1,2,2],
            [1,3,1],
            [3,2,2],
            [2,8,3],
            [2,7,5],
            [2,6,4],
            [2,4,4],
            [4,6,1],
            [7,6,2],
            [4,5,2],
            [8,7,3],
            [8,9,6],
            [9,10,1]
        ];
        
        for i in 0 .. 13
        {
            connections.push(structs::ConnectionStruct{
                source_node: temp_connections[i][0],
                destination_node: temp_connections[i][1],
                weight: temp_connections[i][2]
            });
        }

    }

    // Prints package tracking numbers to verify loading into packages vector
    /*for i in 0 .. packages.len()
    {
        println!("{}", packages[i].tracking_number);
    }*/

    // Prints from connections list source and destination

    println!("Source Node\t->\tDestination Node");
    for i in 0 .. connections.len()
    {
        println!("\t{}\t->\t{}", connections[i].source_node, connections[i].destination_node);
    }

    let route = [1,3,2,4,5,4,6,7,8,9,10];
    
    // Get config JSON from network and apply settings
    println!("{}", get_config()["boxes"][2]);

}

// todo
// 1. Make fn route_sort() that takes in the BoxStructs on the shelves and assigns them places on the shelves
// 2. Make async fn move_x and async fn move_y to simulate motor control