//
//  IPCHandlers.swift
//  SecondBrain
//
//  Handler implementations for IPC events.
//  Provides a concrete implementation of IPCClientDelegate with logging.
//

import Foundation
import os.log

/// Default IPC event handler with logging
class IPCEventHandler: IPCClientDelegate {
    
    private let logger = Logger(subsystem: "com.secondbrain", category: "IPCHandler")
    
    /// Current mode (updated when mode change notifications are received)
    private(set) var currentMode: Mode = .idle
    
    /// Last known uptime (from heartbeat)
    private(set) var lastUptimeMs: UInt64 = 0
    
    /// Current partial dictation text
    private(set) var currentDictation: String = ""
    
    /// Callback for mode changes
    var onModeChanged: ((Mode) -> Void)?
    
    /// Callback for dictation updates
    var onDictationUpdate: ((String, Bool) -> Void)?
    
    /// Callback for heartbeat
    var onHeartbeat: ((UInt64) -> Void)?
    
    /// Callback for disconnect
    var onDisconnect: (() -> Void)?
    
    // MARK: - IPCClientDelegate
    
    func ipcClient(_ client: IPCClient, didReceiveHeartbeat uptimeMs: UInt64) {
        lastUptimeMs = uptimeMs
        
        let uptimeSecs = uptimeMs / 1000
        let hours = uptimeSecs / 3600
        let mins = (uptimeSecs % 3600) / 60
        let secs = uptimeSecs % 60
        
        logger.debug("Heartbeat: uptime \(hours)h \(mins)m \(secs)s")
        onHeartbeat?(uptimeMs)
    }
    
    func ipcClient(_ client: IPCClient, didReceiveDictationPartial text: String, streamId: String) {
        currentDictation = text
        logger.info("Dictation partial [\(streamId)]: \(text)")
        onDictationUpdate?(text, false)
    }
    
    func ipcClient(_ client: IPCClient, didReceiveDictationFinal text: String, streamId: String) {
        currentDictation = text
        logger.info("Dictation final [\(streamId)]: \(text)")
        onDictationUpdate?(text, true)
        
        // Clear after delivering final
        currentDictation = ""
    }
    
    func ipcClient(_ client: IPCClient, didReceiveModeChange mode: Mode) {
        let oldMode = currentMode
        currentMode = mode
        logger.info("Mode changed: \(oldMode.rawValue) -> \(mode.rawValue)")
        onModeChanged?(mode)
    }
    
    func ipcClient(_ client: IPCClient, didReceiveUnknownEvent type: String, payload: [String: AnyCodable]) {
        logger.warning("Unknown event type: \(type), payload: \(payload)")
    }
    
    func ipcClientDidDisconnect(_ client: IPCClient) {
        logger.warning("Disconnected from daemon")
        currentMode = .idle
        currentDictation = ""
        onDisconnect?()
    }
}

// MARK: - Convenience Extensions

extension IPCClient {
    /// Create a client with a default event handler
    static func withDefaultHandler() -> (IPCClient, IPCEventHandler) {
        let client = IPCClient()
        let handler = IPCEventHandler()
        client.delegate = handler
        return (client, handler)
    }
}
