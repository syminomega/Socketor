#!/usr/bin/env python3
"""
Simple TCP client for testing the TCP server functionality
Usage: 
  python tcp_test_client.py [host] [port]  - Use command line arguments
  python tcp_test_client.py               - Interactive mode (prompts for host and port)
"""

import socket
import sys
import time
import threading

def receive_messages(sock):
    """Continuously receive messages from the server"""
    try:
        while True:
            data = sock.recv(1024)
            if not data:
                print("Server closed the connection")
                break
            print(f"Received: {data.decode('utf-8', errors='ignore')}")
    except Exception as e:
        print(f"Error receiving data: {e}")

def main():
    # Default values
    default_host = "127.0.0.1"
    default_port = 8081
    
    # Check if host and port are provided as command line arguments
    if len(sys.argv) > 2:
        host = sys.argv[1]
        port = int(sys.argv[2])
        print(f"Using command line arguments: {host}:{port}")
    else:
        # Interactive input for host
        host_input = input(f"Enter server IP address (default: {default_host}): ").strip()
        host = host_input if host_input else default_host
        
        # Interactive input for port
        while True:
            port_input = input(f"Enter server port (default: {default_port}): ").strip()
            if not port_input:
                port = default_port
                break
            try:
                port = int(port_input)
                if 1 <= port <= 65535:
                    break
                else:
                    print("Port must be between 1 and 65535")
            except ValueError:
                print("Please enter a valid port number")
    
    print(f"Connecting to TCP server at {host}:{port}")
    
    try:
        # Create socket and connect
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        print(f"Connected to {host}:{port}")
        
        # Start receiving thread
        receive_thread = threading.Thread(target=receive_messages, args=(sock,), daemon=True)
        receive_thread.start()
        
        # Send messages
        while True:
            try:
                message = input("Enter message (or 'quit' to exit): ")
                if message.lower() == 'quit':
                    break
                
                if message.startswith("hex:"):
                    # Send hex data
                    hex_data = message[4:].replace(" ", "")
                    try:
                        data = bytes.fromhex(hex_data)
                        sock.send(data)
                        print(f"Sent hex data: {hex_data}")
                    except ValueError:
                        print("Invalid hex format")
                else:
                    # Send text data
                    sock.send(message.encode('utf-8'))
                    print(f"Sent: {message}")
                    
            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"Error sending data: {e}")
                break
                
    except Exception as e:
        print(f"Connection error: {e}")
    finally:
        try:
            sock.close()
            print("Connection closed")
        except:
            pass

if __name__ == "__main__":
    main()
