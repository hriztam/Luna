import SwiftUI
import Combine

class AppState: ObservableObject {
    static let shared = AppState()
    
    @Published var isInputFocused: Bool = false
    
    private init() {}
}
