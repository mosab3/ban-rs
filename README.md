# ban-rs

A Rust-based implementation of Fail2Ban for detecting and preventing malicious activity by banning IP addresses based on configurable rules and log file patterns.

## Features

- **IP Blocking:** Dynamically block IP addresses that repeatedly fail authentication attempts or trigger specified criteria.
- **Configurable Rules:** Define custom rules for detecting suspicious activity.
- **Log Parsing:** Parse log files from various services to extract relevant information.
- **Dynamic Ban Duration:** Adjust ban durations based on offense severity and frequency.

<!-- TODO: -->
<!-- - **Whitelisting:** Exempt trusted IP addresses or ranges from being banned. -->

- **Backend Storage:** Persist ban records using in-memory storage, files, or databases.
- **Asynchronous Processing:** Handle multiple concurrent requests efficiently using async programming.
- **Alerting:** Notify administrators of bans and thresholds via email or other integrations.
- **Integration:** Compatible with system logs, firewalls, and intrusion detection/prevention systems.
- **Documentation and Testing:** Comprehensive documentation and testing for reliability and ease of use.

## Installation

### Prerequisites

- Rust (latest stable version)
- iptables.
- Redis database (Required, for persistent storage)
- Sudo permission.

### Steps

1. Clone the repository:

    ```sh
    git clone https://github.com/mosab3/ban-rs.git
    cd ban-rs
    ```

2. Build the project:

    ```sh
    cargo build --release
    ```

3. Configure your firewall to allow ban-rs to manage IP blocking (specific to your system setup).

## Usage

1. Edit the configuration file to define your rules and settings:

    ```sh
    cp config/example.toml config/config.toml
    vim config/config.toml
    ```

2. Run the ban-rs service:

    ```sh
    cargo run --release
    ```

3. Monitor the logs and activity:

    ```sh
    tail -f logs/ban-rs.log
    ```

## Configuration

1. The configuration files should be in TOML format.

2. When the program is started, if no configuration file is provided, a default configuration file will be generated at `/etc/ban-rs/config.toml`.

3. The generated configuration file will have the following structure (minus the comments):


```toml
[ssh]
enabled = false
port = 22
logpath = "/var/log/auth.log"
maxretry = 3
bantime = 3600
ignoreip = []

[apache2]
enabled = true
port = [80, 442]
logpath = "var/log/apache/access.log"
maxretry = 10
bantime = 600
ignoreip = []

[nginx]
enabled = true
port = [80, 442]
logpath = "/var/log/nginx/access"
maxretry = 10
bantime = 600
ignoreip = []

# Redis configuration.
[redis]

# Type: String
username = "" # The username for Redis authentication.

# Type: String
password = "" # The password for Redis authentication.

# Type: String
host = "127.0.0.1" # The hostname or IP address of the Redis server. Default is 127.0.0.1

# Type: number (u16)
port = 6379 # The port number of the Redis server. Default is 6379

# Type: number (i64)
db = 0 # Database number, Default is 0
```

## Configuration Variables

| Variable | Explanation |
| --- | --- |
| `enabled` | Enable or disable monitoring services. Optional, Default value is based on the active services. |
| `port` | The port to monitor. Default values are (ssh: 22, (apache2, nginx): [80, 442]) |
| `logpath` | The log file path to monitor. |
| `maxretry` | The maximum number of failed attempts before banning an IP. (In seconds) |
| `bantime` | The time in seconds to ban an IP. (In seconds) |
| `ignoreip` | The list of IP addresses to ignore. |

## Contributing

CHeckout [CONTRIBUTING.md](CONTRIBUTING.md)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For any questions, feel free to open an issue or contact the maintainer at [your email address].
