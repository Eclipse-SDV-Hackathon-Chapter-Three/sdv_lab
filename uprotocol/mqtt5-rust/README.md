# Run the publisher
```bash
RUST_LOG=info cargo run --bin mqtt5_publisher
```

# Run the subscriber
```bash
RUST_LOG=info cargo run --bin mqtt5_subscriber
```

# Subscribe to topic(s) using 'mosquitto_sub' and display its information following a specific format passed after -F flag

Filtering only messages from the 'threadx/A/0/2/8001' source in specif
```bash
mosquitto_sub -h 5.196.78.28 -t 'threadx/A/0/2/8001' -v -F '@H:@M:@S | %t | %p'
```

Filtering all the messages from the 'threadx' authority
```bash
mosquitto_sub -h 5.196.78.28 -t 'threadx/+/+/+/+' -v -F '@H:@M:@S | %t | %p'
```

Filtering all the messages from the 'threadx', 'Vehicle_a' and 'sensor_hub' authorities
```bash
mosquitto_sub -h 5.196.78.28 -t 'threadx/+/+/+/+' -t 'Vehicle_a/+/+/+/+' -t 'sensor_hub/+/+/+/+' -v -F '@H:@M:@S | %t | %p'
```