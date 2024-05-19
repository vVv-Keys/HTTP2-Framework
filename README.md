# HTTP/2 Framework
### The HTTP/2 Framework is a powerful tool for testing and analyzing HTTP/2 web servers. It allows you to send custom HTTP/2 requests, control HTTP/2 framing, and analyze server responses, making it an essential tool for developers and security professionals.

# Features

```
HTTP/2 Client: Send custom HTTP/2 requests to web servers.
Fine-tuned Control: Control HTTP/2 framing to simulate various scenarios.
Flexible Configuration: Configure request parameters such as method, headers, body, and timeouts.
Retries and Timeouts: Set retries and timeouts for robust testing.
Secure Communication: Communicate securely over HTTPS using TLS.
Concurrency Control: Control the concurrency level to simulate different load conditions.
Response Analysis: Analyze server responses including status codes, headers, and response bodies.
Command-line Interface: Use a command-line interface for easy interaction and scripting.
Extensible: Extend functionality with custom plugins and modules.
```

Installation
To use the HTTP/2 Framework, follow these steps:

Clone the repository:

```
git clone https://github.com/your_username/http2-framework.git
```
Install Dependencies:
```
cd http2-framework
cargo build --release
```

Run THE EXECUTABLE:

```
./target/release/http2_framework --help
```
# Usage

To send a custom HTTP/2 request, use the following command:

```http2_framework --url https://example.com --method GET --headers "Authorization: Bearer token" --output response.txt```

Replace https://example.com with the URL of the web server you want to test. You can customize the method, headers, body, and output file according to your requirements.

For advanced usage and additional options, refer to the help documentation:

```http2_framework --help```

# Examples

Send a POST request with JSON body: ```http2_framework --url https://example.com/api --method POST --headers "Content-Type: application/json" --body '{"key": "value"}'```

Simulate high concurrency with 1000 requests: ```http2_framework --url https://example.com --concurrency 1000```

Analyze server responses and save to a file:  ```http2_framework --url https://example.com --output responses.txt```

# Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvements, please open an issue or submit a pull request on GitHub.

# License

This project is licensed under the MIT License - see the LICENSE file for details.

# Acknowledgements

Special thanks to the contributors and open-source projects that made this framework possible.

Inspired by: https://github.com/secengjeff/rapidresetclient

