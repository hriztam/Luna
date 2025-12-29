//
//  IPCMessage.swift
//  SecondBrain
//
//  IPC message types using unified envelope format.
//  All messages are newline-delimited JSON (JSONL).
//

import Foundation

// MARK: - Message Kind

/// Classification of IPC messages
enum MessageKind: String, Codable {
    case command
    case event
    case response
    case error
}

// MARK: - Message Types

/// Message type constants matching the Rust daemon
enum MessageType {
    // Commands (Client -> Server)
    static let setMode = "SET_MODE"
    
    // Responses (Server -> Client)
    static let modeChanged = "MODE_CHANGED"
    
    // Errors (Server -> Client)
    static let invalidModeTransition = "INVALID_MODE_TRANSITION"
    static let invalidMode = "INVALID_MODE"
    static let malformedMessage = "MALFORMED_MESSAGE"
    static let unknownCommand = "UNKNOWN_COMMAND"
    
    // Events (Server -> Client)
    static let dictationPartial = "DICTATION_PARTIAL"
    static let dictationFinal = "DICTATION_FINAL"
    static let heartbeat = "HEARTBEAT"
}

// MARK: - Mode

/// Operating modes of the daemon
enum Mode: String, Codable {
    case idle
    case dictation
    case intelligent
    case agent
    
    var displayName: String {
        switch self {
        case .idle: return "Idle"
        case .dictation: return "Dictation"
        case .intelligent: return "Intelligent"
        case .agent: return "Agent"
        }
    }
}

// MARK: - Message Envelope

/// Universal message envelope for IPC communication
/// All messages follow this structure.
struct IPCEnvelope<P: Codable>: Codable {
    let id: String
    let kind: MessageKind
    let type: String
    let payload: P
    
    /// Create a command envelope
    static func command(id: String, type: String, payload: P) -> IPCEnvelope<P> {
        IPCEnvelope(id: id, kind: .command, type: type, payload: payload)
    }
    
    /// Encode the envelope to a JSON string (for sending)
    func toJSONString() throws -> String {
        let encoder = JSONEncoder()
        let data = try encoder.encode(self)
        guard let string = String(data: data, encoding: .utf8) else {
            throw IPCError.encodingFailed
        }
        return string
    }
}

/// Raw envelope for initial parsing (payload as dictionary)
struct RawEnvelope: Decodable {
    let id: String
    let kind: MessageKind
    let type: String
    let payload: [String: AnyCodable]
    
    /// Parse from a JSON line
    static func from(line: String) throws -> RawEnvelope {
        guard let data = line.data(using: .utf8) else {
            throw IPCError.invalidData
        }
        let decoder = JSONDecoder()
        return try decoder.decode(RawEnvelope.self, from: data)
    }
}

// MARK: - Command Payloads

/// Payload for SET_MODE command
struct SetModePayload: Codable {
    let mode: Mode
}

// MARK: - Response Payloads

/// Payload for MODE_CHANGED response
struct ModeChangedPayload: Codable {
    let mode: Mode
}

// MARK: - Error Payloads

/// Payload for INVALID_MODE_TRANSITION error
struct InvalidModeTransitionPayload: Codable {
    let from: Mode
    let to: Mode
}

/// Payload for INVALID_MODE error
struct InvalidModePayload: Codable {
    let provided: String
}

/// Payload for MALFORMED_MESSAGE error
struct MalformedMessagePayload: Codable {
    let reason: String
}

/// Payload for UNKNOWN_COMMAND error
struct UnknownCommandPayload: Codable {
    let command: String
}

// MARK: - Event Payloads

/// Payload for DICTATION_PARTIAL and DICTATION_FINAL events
struct DictationPayload: Codable {
    let text: String
}

/// Payload for HEARTBEAT event
struct HeartbeatPayload: Codable {
    let uptimeMs: UInt64
    
    private enum CodingKeys: String, CodingKey {
        case uptimeMs = "uptime_ms"
    }
}

// MARK: - IPC Errors

enum IPCError: LocalizedError {
    case notConnected
    case connectionFailed(Error)
    case encodingFailed
    case invalidData
    case decodingFailed(Error)
    case unexpectedMessageType(String)
    case daemonError(type: String, payload: [String: AnyCodable])
    
    var errorDescription: String? {
        switch self {
        case .notConnected:
            return "Not connected to daemon"
        case .connectionFailed(let error):
            return "Connection failed: \(error.localizedDescription)"
        case .encodingFailed:
            return "Failed to encode message"
        case .invalidData:
            return "Invalid data received"
        case .decodingFailed(let error):
            return "Failed to decode message: \(error.localizedDescription)"
        case .unexpectedMessageType(let type):
            return "Unexpected message type: \(type)"
        case .daemonError(let type, _):
            return "Daemon error: \(type)"
        }
    }
}

// MARK: - AnyCodable Helper

/// Type-erased Codable for raw payload access
struct AnyCodable: Codable {
    let value: Any
    
    init(_ value: Any) {
        self.value = value
    }
    
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        
        if container.decodeNil() {
            self.value = NSNull()
        } else if let bool = try? container.decode(Bool.self) {
            self.value = bool
        } else if let int = try? container.decode(Int.self) {
            self.value = int
        } else if let double = try? container.decode(Double.self) {
            self.value = double
        } else if let string = try? container.decode(String.self) {
            self.value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            self.value = array.map { $0.value }
        } else if let dict = try? container.decode([String: AnyCodable].self) {
            self.value = dict.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Could not decode AnyCodable"
            )
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        
        switch value {
        case is NSNull:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map { AnyCodable($0) })
        case let dict as [String: Any]:
            try container.encode(dict.mapValues { AnyCodable($0) })
        default:
            throw EncodingError.invalidValue(
                value,
                EncodingError.Context(
                    codingPath: container.codingPath,
                    debugDescription: "Could not encode AnyCodable"
                )
            )
        }
    }
    
    /// Get the value as a specific type
    func as<T>(_ type: T.Type) -> T? {
        return value as? T
    }
}
