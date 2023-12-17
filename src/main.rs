mod structs;

fn main() {
    // Real package Vector
    let mut packages: Vec<structs::BoxStruct> = vec![];

    {   //////////////////////////////// Setup package list ////////////////////////////////
        // xpos 0 .. 1024
        // ypos 0 .. 1024
        // tracking number 100 .. 110
        // destination 1 .. 9
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
    for i in 0 .. connections.len()
    {
        println!("{} -> {}", connections[i].source_node, connections[i].destination_node);
    }

    let route = [1,3,2,4,5,4,6,7,8,9,10];
    

}