#!/usr/bin/env python3
"""
Simple TCP server for testing the TCP client functionality
Usage: 
  python tcp_test_server.py [host] [port]  - Use command line arguments
  python tcp_test_server.py               - Interactive mode (prompts for host and port)
"""

import socket
import sys
import time
import threading

def handle_client(client_socket, client_address):
    """Handle a client connection"""
    print(f"New client connected: {client_address}")
    try:
        # Send welcome message
        welcome_msg = f"Welcome to TCP test server! You are connected from {client_address}\n"
        client_socket.send(welcome_msg.encode('utf-8'))
        
        while True:
            # Receive data from client
            data = client_socket.recv(1024)
            if not data:
                print(f"Client {client_address} disconnected")
                break
                
            received_message = data.decode('utf-8', errors='ignore')
            print(f"Received from {client_address}: {received_message}")
            
            # Echo the message back
            echo_message = f"Echo: {received_message}"
            client_socket.send(echo_message.encode('utf-8'))
            
    except Exception as e:
        print(f"Error handling client {client_address}: {e}")
    finally:
        client_socket.close()
        print(f"Connection with {client_address} closed")

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
                print("Invalid port number. Please enter a valid integer.")
    
    try:
        # Create server socket
        server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        # Bind and listen
        server_socket.bind((host, port))
        server_socket.listen(5)
        
        print(f"TCP test server started on {host}:{port}")
        print("Waiting for connections...")
        print("Press Ctrl+C to stop the server")
        
        while True:
            # Accept client connections
            client_socket, client_address = server_socket.accept()
            
            # Start a new thread to handle the client
            client_thread = threading.Thread(
                target=handle_client, 
                args=(client_socket, client_address)
            )
            client_thread.daemon = True
            client_thread.start()
            
    except KeyboardInterrupt:
        print("\nServer stopped by user")
    except Exception as e:
        print(f"Server error: {e}")
    finally:
        if 'server_socket' in locals():
            server_socket.close()
        print("Server socket closed")

if __name__ == "__main__":
    main()
