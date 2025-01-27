# IPP Sharing
IPP Sharing is a lightweight tool to share local Windows printers drivelessly via IPP protocol. While it is ideal for home users, it may not be as suitable for enterprise environments due to the absence of advanced features such as color management, custom paper sizes, user authentication, and audit logging.

This is a Windows-only tool, of course. For Linux and macOS users, CUPS (Common Unix Printing System) is recommended for IPP-based printer sharing, as it offers a more professional and feature-rich solution.

## Features
- **Lightweight and Easy to Use**: Designed for simplicity and quick setup.
- **Basic Print Ticket Support**: Includes standard media size, orientation, duplex printing, and color mode.
- **Apple AirPrint Compatibility**: Seamlessly integrates with Apple AirPrint for easy printing from Apple devices.
- **Driver-Free Client Setup**: No driver installation or PPD files required on the client side.
- **DNS-SD (Bonjour) Support**: Enables automatic service discovery for printers.

## Getting Started
### Prerequisites
Before using IPP Sharing, ensure the following requirements are met:
- **Operating System**: Windows 10 or later (required for Rust compatibility).
- **Apple Bonjour**: Install Bonjour for service discovery. It is bundled with [iTunes](https://support.apple.com/en-us/HT210384) or [Bonjour Print Services](https://developer.apple.com/bonjour/). Alternatively, download it from [my Shared Files](https://files.alampy.com/Tools%C2%B7%E5%B7%A5%E5%85%B7/Bonjour).

### Step 1: Generate a Self-Signed Certificate
To enable TLS encryption, generate a self-signed certificate using `openssl`. Here’s an example command:

```shell
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out certificate.pem -days 365 -nodes
```

### Step 2: Download or Build the Binary
Download the prebuilt binary or compile it from the source. Place the binary in a directory and create a `config.yaml` file in the same folder. Below is an example configuration:

```yaml
server:
  addr:
    - "[::]:631"
    - "0.0.0.0:631"
  tls:
    # Specify the paths to your self-signed certificate and private key
    cert: "D:/ipp-sharing/certificate.pem"
    key: "D:/ipp-sharing/key.pem"
devices:
  - name: "Print To PDF (IppSharing)"
    info: "Print To PDF (IppSharing)"
    target: "Microsoft Print to PDF"
    # Generate a unique UUID for each printer
    uuid: "b27599fd-800c-409e-afe9-6dbbe11689ac"
    basepath: "/ipp/to_pdf"
    dnssd: true
```

> [!TIP]  
> The `uuid` field serves as a unique identifier for each printer. Generate a new UUID for every printer using [this UUID generator](https://www.uuidgenerator.net/).

### Step 3: Run the Tool
Execute the binary, and the printer should become available on any modern operating system that supports automatic printer discovery.

### Step 4: Configure Firewall (if applicable)
If a firewall is enabled, ensure that incoming connections are allowed on:
- **TCP Port 631** (for IPP)
- **UDP Port 5353** (for DNS-SD/Bonjour)

## Contributions
Contributions are welcome! Feel free to submit a pull request or open an issue if you encounter any problems.

## License
    Copyright (C) 2024-2025 alampy.com

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

This project is licensed under the AGPL-3.0 License. For more details, refer to the [LICENSE](LICENSE) file.

