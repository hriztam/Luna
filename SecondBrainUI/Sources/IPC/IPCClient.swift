//
//  IPCClient.swift
//  SecondBrain
//
//  Unix domain socket client for daemon communication.
//  Uses newline-delimited JSON (JSONL) with persistent connection.
//

import Foundation
import Network
import os.log

/// Client for communicating with the second-brain daemon over Unix domain socket
class IPCClient {
    
    // MARK: - Types
    
    /// Connection state
    enum ConnectionState {
        case disconnected
        case connecting
        case connected
        case failed(Error)
    }
    
    /// Delegate for receiving IPC events
    weak var delegate: IPCClientDelegate?
    
    // MARK: - Properties
    
    private let socketPath: String
    private var connection: NWConnection?
    private let queue = DispatchQueue(label: "com.secondbrain.ipc-client", qos: .userInitiated)
    private let logger = Logger(subsystem: "com.secondbrain", category: "IPCClient")
    
    /// Buffer for incomplete lines
    private var lineBuffer = Data()
    
    /// Current connection state
    private(set) var state: ConnectionState = .disconnected
    
    /// Counter for generating request IDs
    private var requestCounter: UInt64 = 0
    
    /// Pending responses keyed by request ID
    private var pendingRequests: [String: CheckedContinuation<RawEnvelope, Error>] = [:]
    private let pendingLock = NSLock()
    
    // MARK: - Initialization
    
    init() {
        // Socket path matches daemon config
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        self.socketPath = "\(home)/.local/share/second-brain/daemon.sock"
    }
    
    // MARK: - Connection Management
    
    /// Connect to the daemon
    func connect() async throws {
        guard state != .connected else { return }
        
        state = .connecting
        logger.info("Connecting to daemon at \(self.socketPath)")
        
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            let endpoint = NWEndpoint.unix(path: socketPath)
            let parameters = NWParameters.tcp
            parameters.allowLocalEndpointReuse = true
            
            connection = NWConnection(to: endpoint, using: parameters)
            
            var didResume = false
            connection?.stateUpdateHandler = { [weak self] newState in
                guard let self = self, !didResume else { return }
                
                switch newState {
                case .ready:
                    didResume = true
                    self.state = .connected
                    self.logger.info("Connected to daemon")
                    self.startReadLoop()
                    continuation.resume()
                    
                case .failed(let error):
                    didResume = true
                    self.state = .failed(error)
                    self.logger.error("Connection failed: \(error.localizedDescription)")
                    continuation.resume(throwing: IPCError.connectionFailed(error))
                    
                case .cancelled:
                    didResume = true
                    self.state = .disconnected
                    continuation.resume(throwing: IPCError.notConnected)
                    
                case .waiting(let error):
                    self.logger.warning("Connection waiting: \(error.localizedDescription)")
                    
                default:
                    break
                }
            }
            
            connection?.start(queue: queue)
        }
    }
    
    /// Disconnect from the daemon
    func disconnect() {
        logger.info("Disconnecting from daemon")
        connection?.cancel()
        connection = nil
        state = .disconnected
        
        // Cancel all pending requests
        pendingLock.lock()
        let pending = pendingRequests
        pendingRequests.removeAll()
        pendingLock.unlock()
        
        for (_, continuation) in pending {
            continuation.resume(throwing: IPCError.notConnected)
        }
    }
    
    // MARK: - Read Loop
    
    private func startReadLoop() {
        guard let connection = connection else { return }
        
        readData(from: connection)
    }
    
    private func readData(from connection: NWConnection) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            guard let self = self else { return }
            
            if let error = error {
                self.logger.error("Read error: \(error.localizedDescription)")
                self.handleDisconnect()
                return
            }
            
            if let data = data, !data.isEmpty {
                self.processIncomingData(data)
            }
            
            if isComplete {
                self.logger.info("Connection closed by daemon")
                self.handleDisconnect()
                return
            }
            
            // Continue reading
            self.readData(from: connection)
        }
    }
    
    private func processIncomingData(_ data: Data) {
        lineBuffer.append(data)
        
        // Process complete lines
        while let newlineIndex = lineBuffer.firstIndex(of: UInt8(ascii: "\n")) {
            let lineData = lineBuffer[lineBuffer.startIndex..<newlineIndex]
            lineBuffer = lineBuffer[(newlineIndex + 1)...]
            
            if let line = String(data: lineData, encoding: .utf8), !line.isEmpty {
                processLine(line)
            }
        }
    }
    
    private func processLine(_ line: String) {
        logger.debug("Received: \(line)")
        
        // Parse the raw envelope
        let envelope: RawEnvelope
        do {
            envelope = try RawEnvelope.from(line: line)
        } catch {
            logger.error("Failed to parse message: \(error.localizedDescription)")
            return
        }
        
        // Check if this is a response to a pending request
        pendingLock.lock()
        if let continuation = pendingRequests.removeValue(forKey: envelope.id) {
            pendingLock.unlock()
            continuation.resume(returning: envelope)
            return
        }
        pendingLock.unlock()
        
        // Dispatch based on kind
        switch envelope.kind {
        case .event:
            handleEvent(envelope)
        case .response:
            // Orphan response - log it
            logger.warning("Received response with unknown ID: \(envelope.id)")
        case .error:
            // Orphan error - log it
            logger.warning("Received error with unknown ID: \(envelope.id), type: \(envelope.type)")
        case .command:
            // Clients shouldn't receive commands
            logger.warning("Received unexpected command: \(envelope.type)")
        }
    }
    
    private func handleEvent(_ envelope: RawEnvelope) {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            
            switch envelope.type {
            case MessageType.heartbeat:
                if let uptimeMs = envelope.payload["uptime_ms"]?.as(Int.self) {
                    self.delegate?.ipcClient(self, didReceiveHeartbeat: UInt64(uptimeMs))
                }
                
            case MessageType.dictationPartial:
                if let text = envelope.payload["text"]?.as(String.self) {
                    self.delegate?.ipcClient(self, didReceiveDictationPartial: text, streamId: envelope.id)
                }
                
            case MessageType.dictationFinal:
                if let text = envelope.payload["text"]?.as(String.self) {
                    self.delegate?.ipcClient(self, didReceiveDictationFinal: text, streamId: envelope.id)
                }
                
            case MessageType.modeChanged:
                if let modeStr = envelope.payload["mode"]?.as(String.self),
                   let mode = Mode(rawValue: modeStr) {
                    self.delegate?.ipcClient(self, didReceiveModeChange: mode)
                }
                
            default:
                self.logger.info("Unknown event type: \(envelope.type)")
                self.delegate?.ipcClient(self, didReceiveUnknownEvent: envelope.type, payload: envelope.payload)
            }
        }
    }
    
    private func handleDisconnect() {
        state = .disconnected
        connection = nil
        lineBuffer.removeAll()
        
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            self.delegate?.ipcClientDidDisconnect(self)
        }
    }
    
    // MARK: - Sending Messages
    
    /// Generate a unique request ID
    private func nextRequestId() -> String {
        requestCounter += 1
        return "swift-\(requestCounter)"
    }
    
    /// Send a command and wait for a response
    private func sendCommand<P: Codable>(type: String, payload: P) async throws -> RawEnvelope {
        guard let connection = connection, state == .connected else {
            throw IPCError.notConnected
        }
        
        let requestId = nextRequestId()
        let envelope = IPCEnvelope.command(id: requestId, type: type, payload: payload)
        
        // Encode to JSON line
        let jsonString = try envelope.toJSONString()
        let lineData = (jsonString + "\n").data(using: .utf8)!
        
        logger.debug("Sending: \(jsonString)")
        
        // Register for response before sending
        let response: RawEnvelope = try await withCheckedThrowingContinuation { continuation in
            pendingLock.lock()
            pendingRequests[requestId] = continuation
            pendingLock.unlock()
            
            connection.send(content: lineData, completion: .contentProcessed { [weak self] error in
                if let error = error {
                    self?.pendingLock.lock()
                    self?.pendingRequests.removeValue(forKey: requestId)
                    self?.pendingLock.unlock()
                    continuation.resume(throwing: error)
                }
            })
        }
        
        return response
    }
    
    // MARK: - High-Level API
    
    /// Set the daemon mode
    func setMode(_ mode: Mode) async throws {
        let response = try await sendCommand(
            type: MessageType.setMode,
            payload: SetModePayload(mode: mode)
        )
        
        switch response.kind {
        case .response:
            if response.type == MessageType.modeChanged {
                logger.info("Mode changed to \(mode.rawValue)")
            }
        case .error:
            throw IPCError.daemonError(type: response.type, payload: response.payload)
        default:
            throw IPCError.unexpectedMessageType(response.type)
        }
    }
}

// MARK: - Delegate Protocol

/// Protocol for receiving IPC events
protocol IPCClientDelegate: AnyObject {
    /// Called when a heartbeat is received
    func ipcClient(_ client: IPCClient, didReceiveHeartbeat uptimeMs: UInt64)
    
    /// Called when a dictation partial result is received
    func ipcClient(_ client: IPCClient, didReceiveDictationPartial text: String, streamId: String)
    
    /// Called when a dictation final result is received
    func ipcClient(_ client: IPCClient, didReceiveDictationFinal text: String, streamId: String)
    
    /// Called when a mode change notification is received
    func ipcClient(_ client: IPCClient, didReceiveModeChange mode: Mode)
    
    /// Called when an unknown event is received
    func ipcClient(_ client: IPCClient, didReceiveUnknownEvent type: String, payload: [String: AnyCodable])
    
    /// Called when the connection is lost
    func ipcClientDidDisconnect(_ client: IPCClient)
}

// MARK: - Default Delegate Implementations

extension IPCClientDelegate {
    func ipcClient(_ client: IPCClient, didReceiveHeartbeat uptimeMs: UInt64) {}
    func ipcClient(_ client: IPCClient, didReceiveDictationPartial text: String, streamId: String) {}
    func ipcClient(_ client: IPCClient, didReceiveDictationFinal text: String, streamId: String) {}
    func ipcClient(_ client: IPCClient, didReceiveModeChange mode: Mode) {}
    func ipcClient(_ client: IPCClient, didReceiveUnknownEvent type: String, payload: [String: AnyCodable]) {}
    func ipcClientDidDisconnect(_ client: IPCClient) {}
}
