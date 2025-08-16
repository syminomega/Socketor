#!/usr/bin/env python3
import socket
import threading
import time

def udp_server():
    """Simple UDP server for testing"""
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    server_socket.bind(('127.0.0.1', 8082))
    print("UDP Server started on 127.0.0.1:8082")
    
    try:
        while True:
            data, client_address = server_socket.recvfrom(1024)
            message = data.decode('utf-8')
            print(f"Received from {client_address}: {message}")
            
            # Echo the message back
            response = f"Echo: {message}"
            server_socket.sendto(response.encode('utf-8'), client_address)
            print(f"Sent to {client_address}: {response}")
            
    except KeyboardInterrupt:
        print("\nShutting down UDP server...")
    finally:
        server_socket.close()

if __name__ == "__main__":
    udp_server()
