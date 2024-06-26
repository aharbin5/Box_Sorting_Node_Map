import socket

hostname = "192.168.10.10"
port = 48753
message = """
{"route": [
	{"tracking_number": 4001},
	{"tracking_number": 6008},
	{"tracking_number": 8010},
    {"tracking_number": 6005},
    {"tracking_number": 4003},
    {"tracking_number": 8012},
    {"tracking_number": 6007},
    {"tracking_number": 8009},
    {"tracking_number": 4002},
    {"tracking_number": 4004},
    {"tracking_number": 8011},
    {"tracking_number": 6006}]}"""

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
print("Socket Initialized")
s.connect((hostname, port))
print("Connected to hostname and port")
s.sendall((message.encode('utf-8')))
print("Sent, closing port")
s.close()