import SwiftUI
import AppKit
import KeyboardShortcuts

class AppDelegate: NSObject, NSApplicationDelegate {
    // The status item in the menu bar
    var statusItem: NSStatusItem?
    // The popover that appears when clicking the status item
    var popover = NSPopover()
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Create the SwiftUI view that provides the window contents.
        let contentView = ContentView()
        
        // Set the SwiftUI view as the content of the popover
        popover.contentSize = NSSize(width: 200, height: 100)
        popover.behavior = .transient // Closes when clicking outside
        popover.contentViewController = NSHostingController(rootView: contentView)
        
        // Create the status item
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        
        // Configure the status item button
        if let button = statusItem?.button {
            // Use a system symbol for the icon
            button.image = NSImage(systemSymbolName: "moon.stars.fill", accessibilityDescription: "Luna")
            button.action = #selector(togglePopover(_:))
        }
        
        // Register global hotkey listener
        KeyboardShortcuts.onKeyUp(for: .invokeLuna) { [weak self] in
            self?.togglePopover(nil)
        }
    }
    
    @objc func togglePopover(_ sender: AnyObject?) {
        if let button = statusItem?.button {
            if popover.isShown {
                popover.performClose(sender)
            } else {
                popover.show(relativeTo: button.bounds, of: button, preferredEdge: NSRectEdge.minY)
                // Bring app to front so the popover is active
                NSApp.activate(ignoringOtherApps: true)
                // Request focus for the input field
                AppState.shared.focusInputSubject.send()
            }
        }
    }
}
