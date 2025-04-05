# Holz Logistik Server

A Rust implementation of a real-time synchronization server for the Holz Logistik app.

## Features

- WebSocket-based real-time synchronization
- User authentication via API keys
- User management (CRUD operations)
- SQLite database storage
- Type-safe communication with the mobile app

## Architecture

The server is designed to work with the Holz Logistik Flutter app for real-time synchronization of user data. It provides:

- WebSocket endpoint for real-time communication
- REST API endpoints for authentication and user management
- SQLite database for data persistence

## API Documentation

### WebSocket Endpoint

- `GET /ws` - WebSocket connection endpoint

### Authentication Endpoints

- `POST /api/auth` - Authenticate with API key
- `POST /api/auth/api-key` - Generate a new API key for a user

### User Management Endpoints

- `GET /api/users` - Get all users
- `POST /api/users` - Create or update a user
- `GET /api/users/:id` - Get a specific user
- `DELETE /api/users/:id` - Delete a user

## WebSocket Protocol

The WebSocket protocol uses JSON messages with the following structure:

```json
{
  "type": "message_type",
  "data": {},
  "timestamp": "2023-05-01T12:34:56Z",
  "id": "unique-uuid-v4"
}
```

### Supported Message Types

- `authentication_request` - Client requests authentication
- `authentication_response` - Server responds to authentication
- `user_update` - User data update (both directions)
- `user_deletion` - User deletion notification
- `connection_status` - Connection status update
- `ping` / `pong` - Heartbeat messages

## Connecting from the Flutter App

Update the Flutter app's WebSocket URL in `lib/bootstrap.dart`:

```dart
const url = 'ws://your-server-ip:8080/ws';
```

## License

MIT