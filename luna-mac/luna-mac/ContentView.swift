import SwiftUI
import Combine


struct ContentView: View {
    @State private var commandText: String = ""
    @State private var statusMessage: String?
    @State private var statusIsError: Bool = false
    @State private var recentCommands: [String] = ["help", "version", "clear"] // Mock data
    
    // Observe global app state for focus requests
    @ObservedObject var appState = AppState.shared
    
    // Focus state for the text field
    @FocusState private var isInputFocused: Bool
    
    var body: some View {
        ZStack(alignment: .bottom) {
            VStack(spacing: 16) {
                // Header / Title
                HStack {
                    Image(systemName: "moon.stars.fill")
                        .foregroundColor(.accentColor)
                    Text("Luna Command")
                        .font(.headline)
                    Spacer()
                }
                .padding(.horizontal)
                .padding(.top)
                
                // Input Area
                HStack {
                    TextField("Enter command...", text: $commandText)
                        .textFieldStyle(PlainTextFieldStyle())
                        .padding(10)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(8)
                        .overlay(
                            RoundedRectangle(cornerRadius: 8)
                                .stroke(Color.gray.opacity(0.3), lineWidth: 1)
                        )
                        .focused($isInputFocused)
                        .onSubmit {
                            runCommand()
                        }
                    
                    Button(action: runCommand) {
                        Text("Run")
                            .fontWeight(.semibold)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 8)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.regular)
                }
                .padding(.horizontal)
                
                // Recent Commands Section
                VStack(alignment: .leading, spacing: 8) {
                    Text("Recent")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .padding(.horizontal)
                    
                    ScrollView {
                        VStack(alignment: .leading, spacing: 4) {
                            ForEach(recentCommands, id: \.self) { cmd in
                                Button(action: {
                                    commandText = cmd
                                    isInputFocused = true
                                }) {
                                    HStack {
                                        Image(systemName: "clock")
                                            .font(.caption2)
                                        Text(cmd)
                                            .font(.system(.body, design: .monospaced))
                                        Spacer()
                                    }
                                    .padding(.horizontal)
                                    .padding(.vertical, 4)
                                    .contentShape(Rectangle())
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                    .frame(maxHeight: 100)
                }
                
                Spacer()
            }
            
            // Toast Notification
            if let message = statusMessage {
                Text(message)
                    .font(.caption)
                    .foregroundColor(.white)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(
                        Capsule()
                            .fill(statusIsError ? Color.red.opacity(0.9) : Color.green.opacity(0.9))
                    )
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                    .padding(.bottom, 20)
                    .onAppear {
                        // Auto-hide after 2 seconds
                        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                            withAnimation {
                                statusMessage = nil
                            }
                        }
                    }
            }
        }
        .frame(width: 400, height: 300)
        .preferredColorScheme(.dark)
        .onAppear {
            isInputFocused = true
        }
        // Sync external focus request to local FocusState
        .onReceive(appState.focusInputSubject) { _ in
            // Delay slightly to ensure window is active
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                isInputFocused = true
            }
        }
    }
    
    private func runCommand() {
        guard !commandText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        
        let cmd = commandText
        
        // Mock execution logic
        print("Running command: \(cmd)")
        
        // Add to recent commands (avoid duplicates at top)
        if let index = recentCommands.firstIndex(of: cmd) {
            recentCommands.remove(at: index)
        }
        recentCommands.insert(cmd, at: 0)
        if recentCommands.count > 5 {
            recentCommands.removeLast()
        }
        
        // Clear input
        commandText = ""
        
        // Show success toast
        withAnimation {
            statusIsError = false
            statusMessage = "Command received: \(cmd)"
        }
        
        // Keep focus
        isInputFocused = true
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
