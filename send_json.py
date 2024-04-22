import socket

hostname = "127.0.0.1"
port = 48753
message = """
{"route": [
	{"destination_id": 201, "x": 0, "y": 0}, 
	{"destination_id": 202, "x": 10, "y": 0},
	{"destination_id": 203, "x": 0, "y": 10},
	{"destination_id": 204, "x": 10, "y": 10},
	{"destination_id": 205, "x": 20, "y": 10}],
"box_loc": [
	{"tracking_number": 101, "destination": 0},
	{"tracking_number": 102, "destination": 1},
	{"tracking_number": 103, "destination": 1},
	{"tracking_number": 104, "destination": 2},
	{"tracking_number": 105, "destination": 3},
	{"tracking_number": 106, "destination": 3}]}"""

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect((hostname, port))
s.sendall((message.encode('utf-8')))
s.close()