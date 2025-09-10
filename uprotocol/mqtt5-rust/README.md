# Run the publisher
```bash
RUST_LOG=info cargo run
```

# Subscribe to the topic and display its information following a specific format passed after -F flag
```bash
mosquitto_sub -h 5.196.78.28 -t 'threadx/A/0/2/8001' -v -F '@H:@M:@S | %t | %p'
```
