import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers.dart';
import '../rust/api.dart' as api;
import '../rust/frb_mirrors.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage> {
  final _syncUrlController = TextEditingController();
  bool _syncing = false;

  @override
  void dispose() {
    _syncUrlController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final settings = ref.watch(settingsProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: settings.when(
        data: (doc) => _SettingsBody(
          doc: doc,
          syncUrlController: _syncUrlController,
          syncing: _syncing,
          onSave: (updated) async {
            await ref.read(settingsProvider.notifier).save(updated);
            if (context.mounted) {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Settings saved')),
              );
            }
          },
          onStartSync: () async {
            final url = _syncUrlController.text.trim();
            if (url.isEmpty) return;
            setState(() => _syncing = true);
            try {
              await api.startSync(url: url);
            } catch (e) {
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Sync error: $e')),
                );
              }
            }
          },
          onStopSync: () async {
            await api.stopSync();
            setState(() => _syncing = false);
          },
        ),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('Error: $e')),
      ),
    );
  }
}

class _SettingsBody extends StatefulWidget {
  final SettingsDocument doc;
  final TextEditingController syncUrlController;
  final bool syncing;
  final Future<void> Function(SettingsDocument) onSave;
  final VoidCallback onStartSync;
  final VoidCallback onStopSync;

  const _SettingsBody({
    required this.doc,
    required this.syncUrlController,
    required this.syncing,
    required this.onSave,
    required this.onStartSync,
    required this.onStopSync,
  });

  @override
  State<_SettingsBody> createState() => _SettingsBodyState();
}

class _SettingsBodyState extends State<_SettingsBody> {
  late TextEditingController _authorController;
  late String _volumeUnit;
  late String _massUnit;
  late String _tempUnit;

  @override
  void initState() {
    super.initState();
    _authorController = TextEditingController(text: widget.doc.defaultAuthor);
    _volumeUnit = widget.doc.defaultVolumeUnit;
    _massUnit = widget.doc.defaultMassUnit;
    _tempUnit = widget.doc.defaultTemperatureUnit;
  }

  @override
  void dispose() {
    _authorController.dispose();
    super.dispose();
  }

  SettingsDocument _buildDoc() {
    return SettingsDocument(
      id: widget.doc.id,
      vimMode: widget.doc.vimMode,
      defaultVolumeUnit: _volumeUnit,
      defaultMassUnit: _massUnit,
      defaultTemperatureUnit: _tempUnit,
      openaiApiKey: widget.doc.openaiApiKey,
      defaultAuthor: _authorController.text,
      aiBaseUrl: widget.doc.aiBaseUrl,
      aiModel: widget.doc.aiModel,
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        TextField(
          controller: _authorController,
          decoration: const InputDecoration(
            labelText: 'Default Author',
            border: OutlineInputBorder(),
          ),
        ),
        const SizedBox(height: 16),
        DropdownButtonFormField<String>(
          initialValue: _volumeUnit,
          decoration: const InputDecoration(
            labelText: 'Volume Unit',
            border: OutlineInputBorder(),
          ),
          items: const [
            DropdownMenuItem(value: 'gal', child: Text('Gallons')),
            DropdownMenuItem(value: 'l', child: Text('Liters')),
          ],
          onChanged: (v) => setState(() => _volumeUnit = v!),
        ),
        const SizedBox(height: 16),
        DropdownButtonFormField<String>(
          initialValue: _massUnit,
          decoration: const InputDecoration(
            labelText: 'Mass Unit',
            border: OutlineInputBorder(),
          ),
          items: const [
            DropdownMenuItem(value: 'lb', child: Text('Pounds')),
            DropdownMenuItem(value: 'kg', child: Text('Kilograms')),
          ],
          onChanged: (v) => setState(() => _massUnit = v!),
        ),
        const SizedBox(height: 16),
        DropdownButtonFormField<String>(
          initialValue: _tempUnit,
          decoration: const InputDecoration(
            labelText: 'Temperature Unit',
            border: OutlineInputBorder(),
          ),
          items: const [
            DropdownMenuItem(value: 'F', child: Text('Fahrenheit')),
            DropdownMenuItem(value: 'C', child: Text('Celsius')),
          ],
          onChanged: (v) => setState(() => _tempUnit = v!),
        ),
        const SizedBox(height: 24),
        FilledButton(
          onPressed: () => widget.onSave(_buildDoc()),
          child: const Text('Save Settings'),
        ),
        const Divider(height: 48),
        Text('Sync', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: 16),
        TextField(
          controller: widget.syncUrlController,
          decoration: const InputDecoration(
            labelText: 'Sync Server URL',
            hintText: 'wss://sync.example.com',
            border: OutlineInputBorder(),
          ),
        ),
        const SizedBox(height: 16),
        Row(
          children: [
            FilledButton(
              onPressed: widget.syncing ? null : widget.onStartSync,
              child: const Text('Start Sync'),
            ),
            const SizedBox(width: 16),
            OutlinedButton(
              onPressed: widget.syncing ? widget.onStopSync : null,
              child: const Text('Stop Sync'),
            ),
            if (widget.syncing) ...[
              const SizedBox(width: 16),
              const SizedBox(
                width: 16,
                height: 16,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
              const SizedBox(width: 8),
              const Text('Syncing...'),
            ],
          ],
        ),
      ],
    );
  }
}
