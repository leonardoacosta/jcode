import JCodeKit
import SwiftUI

/// Settings: the rarely-touched configuration surface, reached from a row inside
/// `SessionsView`.
///
/// Frequent decisions moved out of here on purpose: session switching lives in
/// `SessionsView` (top-left of chat) and model/reasoning in `ModelPickerView`
/// (the header model label). What remains is the low-frequency, look-it-up work.
struct SettingsView: View {
    @Environment(AppModel.self) private var model

    /// Reasoning effort levels offered when the provider exposes the knob.
    /// Owned here (rather than in the picker) because it is the canonical list
    /// of provider-supported values.
    static let reasoningEfforts = ["none", "low", "medium", "high", "xhigh"]

    var body: some View {
        NavigationStack {
            List {
                modelSection
                if model.session.reasoningEffort != nil {
                    reasoningSection
                }
                SettingsSessionsSection(renameDraft: $renameDraft, showRename: $showRename)
                SettingsServersSection(showPairNew: $showPairNew)
                SettingsInfoSection()
            }
            .scrollContentBackground(.hidden)
            .background(Theme.background)
            .listStyle(.insetGrouped)
            .listRowSeparatorTint(Theme.border)
            .dynamicTypeSize(.large ... .accessibility3)
            .navigationTitle("Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
        .preferredColorScheme(.dark)
        .alert("Rename session", isPresented: $showRename) {
            TextField("Title", text: $renameDraft)
            Button("Rename") {
                model.renameSession(renameDraft)
            }
            Button("Cancel", role: .cancel) {}
        }
        .sheet(isPresented: $showPairNew) {
            NavigationStack {
                PairingView()
                    .background(Theme.background)
                    .toolbar {
                        ToolbarItem(placement: .cancellationAction) {
                            Button("Cancel") { showPairNew = false }
                        }
                    }
            }
            .preferredColorScheme(.dark)
        }
        .onChange(of: model.activeServer?.id) {
            showPairNew = false
        }
    }

    private var modelSection: some View {
        Section("Model") {
            ForEach(model.session.availableModels, id: \.self) { name in
                let isActive = name == model.session.modelName
                Button {
                    model.setModel(name)
                } label: {
                    HStack {
                        Text(name)
                            .font(Theme.mono(13))
                            .foregroundStyle(Theme.textPrimary)
                            .lineLimit(1)
                        Spacer()
                        if isActive {
                            Image(systemName: "checkmark")
                                .font(.caption)
                                .foregroundStyle(Theme.mint)
                                .accessibilityHidden(true)
                        }
                    }
                }
                .listRowBackground(Theme.surface)
                .accessibilityLabel("Model \(name)")
                .accessibilityValue(isActive ? "Selected" : "")
                .accessibilityHint("Selects this model")
                .accessibilityAddTraits(isActive ? [.isSelected] : [])
            }
            SettingsInfoSection()
        }
        .scrollContentBackground(.hidden)
        .background(Theme.background)
        .dynamicTypeSize(.large ... .accessibility3)
        .navigationTitle("Settings")
        .navigationBarTitleDisplayMode(.inline)
    }

    private var reasoningSection: some View {
        Section("Reasoning effort") {
            ForEach(Self.reasoningEfforts, id: \.self) { effort in
                let isActive = effort == model.session.reasoningEffort
                Button {
                    model.setReasoningEffort(effort)
                } label: {
                    HStack {
                        Text(effort)
                            .font(Theme.mono(13))
                            .foregroundStyle(Theme.textPrimary)
                        Spacer()
                        if isActive {
                            Image(systemName: "checkmark")
                                .font(.caption)
                                .foregroundStyle(Theme.mint)
                                .accessibilityHidden(true)
                        }
                    }
                }
                .listRowBackground(Theme.surface)
                .accessibilityLabel("Reasoning effort \(effort)")
                .accessibilityValue(isActive ? "Selected" : "")
                .accessibilityHint("Sets how much the model reasons before answering")
                .accessibilityAddTraits(isActive ? [.isSelected] : [])
            }
        }
    }
}
