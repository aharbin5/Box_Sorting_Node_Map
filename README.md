# Box_Sorting_Node_Map

Rust implementation of PWM motor control, as5600 encoders, and a barcode scanner to sort packages on a shelf.

### Description

This is part of my Interdisciplinary Senior Design Project of a box sorting and retieval robot for the back of delivery box trucks.
This application should,
- Take in a route packet from a network interface
- Create a boxmap on the physical shelf with barcode scanner data
- Deliver box from a position on the shelf to the pedestal

### Usage

Senior design expo is Monday (4/29), if I have time to update this before then I will.
Otherwise, it'll be updated after.

#### Barcodes Accepted

The barcode scanner and theoretically read any barcodes and input them with a position but it will only sort the ones that match both
the route and the ones its scanned.  So here are the ones I'm using for now,
0004001, 0004002, 0004003, 0004004, 0006005, 0006006, 0006007, 0006008, 0008009, 0008010, 0008011, 0008012 :: These are box IDs themselves


### License

Licensed under 
Apache License Version 2.0 https://apache.org/licenses/LICENSE-2.0