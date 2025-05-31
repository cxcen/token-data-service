# Token Data Service

A high-performance, real-time data service for meme token trading platforms. This service provides K-line (candlestick) chart data and real-time transaction streaming capabilities.

## Features

- Real-time K-line data with multiple time intervals (1s, 1m, 5m, 15m, 1h)
- WebSocket-based live transaction streaming
- Real-time K-line updates with "open" bars
- Mock data generation for testing and development
- Support for multiple tokens
- Automatic connection health monitoring with ping/pong
- Scalable architecture with broadcast channels

## Architecture Overview

The service is built with a modular architecture consisting of several key components:

1. **Data Service**
   - Central service managing all real-time data flows
   - Handles both transaction and K-line data broadcasting
   - Maintains in-memory state with thread-safe data structures
   - Uses broadcast channels for efficient real-time updates

2. **WebSocket Layer**
   - Dedicated modules for transaction and K-line streaming
   - Automatic connection health monitoring
   - Efficient message broadcasting with backpressure handling
   - Symbol-based filtering for targeted data delivery

3. **API Layer**
   - REST endpoints for K-line data retrieval
   - WebSocket endpoints for real-time data streaming
   - Health check and monitoring endpoints

4. **Mock Data Generator**
   - Simulates realistic market data
   - Configurable parameters for testing different scenarios
   - Supports continuous data generation

## API Documentation

### REST Endpoints

#### Get K-line Data
```
GET /api/v1/klines/{token_symbol}
```
Query Parameters:
- `interval`: Time interval (1s, 1m, 5m, 15m, 1h)
- `limit`: Number of candles to return (default: 100)
- `from`: Start timestamp (optional)
- `to`: End timestamp (optional)

### WebSocket Endpoints

#### Transaction Stream
```
WS /ws/transactions/{token_symbol}
```
Streams real-time transaction data for the specified token.

Message Format:
```json
{
    "type": "transaction",
    "data": {
        "symbol": "DOGE",
        "price": "0.123",
        "volume": "100.0",
        "side": "Buy",
        "timestamp": "2024-03-21T10:30:00Z"
    }
}
```

Features:
- Real-time transaction updates
- Symbol-based filtering
- Automatic connection health monitoring
- Graceful connection handling

#### Live K-line Updates
```
WS /ws/klines/{token_symbol}/{interval}
```
Streams real-time K-line updates including "open" bars.

Message Format:
```json
{
    "type": "kline",
    "data": {
        "symbol": "DOGE",
        "interval": "1m",
        "open_time": "2024-03-21T10:30:00Z",
        "close_time": "2024-03-21T10:31:00Z",
        "open": "0.123",
        "high": "0.125",
        "low": "0.122",
        "close": "0.124",
        "volume": "1000.0"
    }
}
```

## Setup Instructions

1. Prerequisites
   - Rust 1.70 or higher
   - Cargo package manager

2. Installation
   ```bash
   git clone https://github.com/cxcen/token-data-service.git
   cd token-data-service
   cargo build --release
   ```

3. Running the Service
   ```bash
   cargo run --release -p data-service
   ```

3. Running the Client
   ```bash
   cargo run --release -p ws-client
   ```


4. Running Tests
   ```bash
   cargo test
   ```

## Configuration

The service can be configured through environment variables:

- `RUST_LOG`: Log level (info, debug, trace)
- `PORT`: HTTP server port (default: 8080)
- `BROADCAST_CHANNEL_SIZE`: Size of broadcast channels for real-time data (default: 1000)
- `MAX_HISTORY`: Maximum number of historical K-lines to keep in memory (default: 1000)

## WebSocket Connection Management

The service implements robust WebSocket connection management:

1. **Health Monitoring**
   - Automatic ping messages every 5 seconds
   - Connection timeout after 10 seconds of no pong response
   - Graceful connection cleanup on timeout or errors

2. **Backpressure Handling**
   - Efficient broadcast channels with configurable capacity
   - Automatic cleanup of slow consumers
   - Memory-efficient message passing

3. **Error Handling**
   - Graceful error recovery
   - Automatic reconnection support
   - Detailed error logging

## Assumptions

1. Data Persistence: The service currently maintains data in-memory and does not persist historical data
2. Mock Data: The service uses simulated data instead of real market data
3. Authentication: The service assumes authentication is handled by an API gateway

## Future Improvements

1. Data Persistence Layer
   - Integration with a time-series database
   - Historical data API endpoints

2. Performance Optimizations
   - Implement data compression
   - Add request rate limiting
   - Optimize memory usage for long-running sessions

3. Additional Features
   - Market depth data
   - Order book visualization support
   - Technical indicators calculation
   - Multiple exchange support

4. Production Readiness
   - Metrics and monitoring
   - Circuit breakers
   - Rate limiting
   - Authentication and authorization
   - Docker containerization
