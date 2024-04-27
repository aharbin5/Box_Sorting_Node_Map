import socket

hostname = "192.168.10.10"
port = 48753
message = """
{"box_loc": [
	{"tracking_number": 4004},
	{"tracking_number": 6008},
	{"tracking_number": 4002}]}"""

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect((hostname, port))
s.sendall((message.encode('utf-8')))
s.close()