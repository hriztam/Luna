import SwiftUI
import Combine

class AppState: ObservableObject {
    static let shared = AppState()
    
    @Published var isInputFocused: Bool = false
    // A subject to trigger focus programmatically (e.g. from hotkey)
    let focusInputSubject = PassthroughSubject<Void, Never>()
    
    private init() {}
}
