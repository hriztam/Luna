import SwiftUI
import KeyboardShortcuts

extension KeyboardShortcuts.Name {
    static let invokeLuna = Self("invokeLuna", default: .init(.space, modifiers: [.control]))
}

@main
struct luna_macApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var body: some Scene {
        Settings {
            EmptyView()
        }
    }
}
