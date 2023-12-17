# Box_Sorting_Node_Map

Rust node map for organizing and sorting package delivery

### Description

This is part of my Interdisciplinary Senior Design Project of a box sorting and retieval robot for the back of delivery box trucks.
This application should,
- Take in a route packet from a network interface
- Either have a pre-loaded box map or create one from barcode data
- Organize boxes from closest stop to farthest stop on the shelves
- Move those boxes using multithreading
- Decide if there is enough time to move a box between stops

### Usage

TBA

### main

A Vector of BoxStruct structs are created with testing data
An arbitrary nodemap with connections and weights is created
A route is created but unused for now

### structs

#### BoxStruct

This struct is used to organize box information without having to remember what BoxStruct[0] is vs BoxStruct[2]

#### ConnectionStruct

Same thing as BoxStruct but it stores connection data between two nodes

### License

Licensed under 
Apache License Version 2.0 https://apache.org/licenses/LICENSE-2.0