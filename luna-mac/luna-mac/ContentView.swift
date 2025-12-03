import SwiftUI

struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "moon.stars.fill")
                .imageScale(.large)
                .foregroundColor(.accentColor)
            Text("Hello from Luna")
                .font(.headline)
                .padding(.top, 5)
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        // Force dark mode appearance for the popover content
        .preferredColorScheme(.dark)
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
