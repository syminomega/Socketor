#!/usr/bin/env python3
import socket
import time

def udp_client():
    """Simple UDP client for testing"""
    client_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    server_address = ('127.0.0.1', 8082)
    
    try:
        # Send a test message
        message = "Hello from Python UDP client!"
        print(f"Sending to {server_address}: {message}")
        client_socket.sendto(message.encode('utf-8'), server_address)
        
        # Receive the response
        data, server = client_socket.recvfrom(1024)
        response = data.decode('utf-8')
        print(f"Received from {server}: {response}")
        
    except Exception as e:
        print(f"Error: {e}")
    finally:
        client_socket.close()

if __name__ == "__main__":
    udp_client()
