# Meme Token Data Service

A high-performance, real-time data service for meme token trading platforms. This service provides K-line (candlestick) chart data and real-time transaction streaming capabilities.

## Features

- Real-time K-line data with multiple time intervals (1s, 1m, 5m, 15m, 1h)
- WebSocket-based live transaction streaming
- Mock data generation for testing and development
- Support for multiple tokens
- Real-time updating "open" K-line bars
- Scalable architecture

## Architecture Overview

The service is built with a modular architecture consisting of several key components:

1. **Data Engine**
   - Manages real-time price and transaction data
   - Generates and updates K-line data for different time intervals
   - Maintains in-memory state with thread-safe data structures

2. **API Layer**
   - REST endpoints for K-line data retrieval
   - WebSocket endpoints for real-time data streaming
   - Health check and monitoring endpoints

3. **Mock Data Generator**
   - Simulates realistic market data
   - Configurable parameters for testing different scenarios

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

#### Live K-line Updates
```
WS /ws/klines/{token_symbol}/{interval}
```
Streams real-time K-line updates including "open" bars.

## Setup Instructions

1. Prerequisites
   - Rust 1.70 or higher
   - Cargo package manager

2. Installation
   ```bash
   git clone https://github.com/your-org/meme-token-service.git
   cd meme-token-service
   cargo build --release
   ```

3. Running the Service
   ```bash
   cargo run --release
   ```

4. Running Tests
   ```bash
   cargo test
   ```

## Configuration

The service can be configured through environment variables:

- `RUST_LOG`: Log level (info, debug, trace)
- `PORT`: HTTP server port (default: 8080)
- `MOCK_DATA_INTERVAL`: Interval for mock data generation in milliseconds
- `MAX_CONNECTIONS`: Maximum number of WebSocket connections

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

## License

MIT License 