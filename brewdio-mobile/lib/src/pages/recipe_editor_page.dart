import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers.dart';
import '../rust/api.dart' as api;
import '../rust/frb_mirrors.dart';
import 'home_page.dart' show volumeUnitLabel;

class RecipeEditorPage extends ConsumerStatefulWidget {
  final Recipe recipe;
  const RecipeEditorPage({super.key, required this.recipe});

  @override
  ConsumerState<RecipeEditorPage> createState() => _RecipeEditorPageState();
}

class _RecipeEditorPageState extends ConsumerState<RecipeEditorPage> {
  late Recipe _recipe;
  late TextEditingController _nameController;
  api.RecipeCalculations? _calcs;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _recipe = widget.recipe;
    _nameController = TextEditingController(text: _recipe.name);
    _loadCalcs();
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _loadCalcs() async {
    final calcs = await api.calculateRecipe(recipe: _recipe.recipe);
    if (mounted) setState(() => _calcs = calcs);
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    try {
      await api.updateRecipe(
        id: _recipe.id,
        name: _nameController.text,
        recipe: _recipe.recipe,
        equipment: _recipe.equipment,
      );
      ref.read(recipeListProvider.notifier).refresh();
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 5,
      child: Scaffold(
        appBar: AppBar(
          title: Text(_nameController.text),
          actions: [
            if (_saving)
              const Padding(
                padding: EdgeInsets.all(16),
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              )
            else
              IconButton(
                icon: const Icon(Icons.save),
                onPressed: _save,
              ),
          ],
          bottom: const TabBar(
            isScrollable: true,
            tabs: [
              Tab(text: 'Overview'),
              Tab(text: 'Fermentables'),
              Tab(text: 'Hops'),
              Tab(text: 'Cultures'),
              Tab(text: 'Mash'),
            ],
          ),
        ),
        body: TabBarView(
          children: [
            _OverviewTab(
              recipe: _recipe,
              calcs: _calcs,
              nameController: _nameController,
              onNameChanged: () => setState(() {}),
            ),
            _FermentablesTab(recipe: _recipe.recipe),
            _HopsTab(recipe: _recipe.recipe),
            _CulturesTab(recipe: _recipe.recipe),
            _MashTab(recipe: _recipe.recipe),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Overview Tab
// ---------------------------------------------------------------------------

class _OverviewTab extends StatelessWidget {
  final Recipe recipe;
  final api.RecipeCalculations? calcs;
  final TextEditingController nameController;
  final VoidCallback onNameChanged;

  const _OverviewTab({
    required this.recipe,
    required this.calcs,
    required this.nameController,
    required this.onNameChanged,
  });

  @override
  Widget build(BuildContext context) {
    final r = recipe.recipe;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        TextField(
          controller: nameController,
          decoration: const InputDecoration(
            labelText: 'Recipe Name',
            border: OutlineInputBorder(),
          ),
          onChanged: (_) => onNameChanged(),
        ),
        const SizedBox(height: 16),
        if (r.style != null)
          _InfoRow('Style', r.style!.field0.name),
        _InfoRow('Type', r.type.name),
        _InfoRow(
          'Batch Size',
          '${r.batchSize.value} ${volumeUnitLabel(r.batchSize.unit)}',
        ),
        _InfoRow(
          'Efficiency',
          '${r.efficiency.brewhouse.value}%',
        ),
        const Divider(height: 32),
        if (calcs != null) ...[
          Text('Calculations',
              style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          _CalcRow(calcs!),
        ],
        if (r.notes != null && r.notes!.isNotEmpty) ...[
          const Divider(height: 32),
          Text('Notes', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text(r.notes!),
        ],
      ],
    );
  }
}

class _InfoRow extends StatelessWidget {
  final String label;
  final String value;
  const _InfoRow(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 120,
            child: Text(label, style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            )),
          ),
          Text(value, style: Theme.of(context).textTheme.bodyLarge),
        ],
      ),
    );
  }
}

class _CalcRow extends StatelessWidget {
  final api.RecipeCalculations calcs;
  const _CalcRow(this.calcs);

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 16,
      runSpacing: 8,
      children: [
        _StatChip('OG', calcs.og.toStringAsFixed(3)),
        _StatChip('FG', calcs.fg.toStringAsFixed(3)),
        _StatChip('ABV', '${calcs.abv.toStringAsFixed(1)}%'),
        _StatChip('IBU', calcs.ibu.toStringAsFixed(0)),
        _StatChip('SRM', calcs.srm.toStringAsFixed(1),
            color: _parseColor(calcs.colorHex)),
      ],
    );
  }

  Color? _parseColor(String hex) {
    final h = hex.replaceAll('#', '');
    if (h.length == 6) {
      return Color(int.parse('FF$h', radix: 16));
    }
    return null;
  }
}

class _StatChip extends StatelessWidget {
  final String label;
  final String value;
  final Color? color;
  const _StatChip(this.label, this.value, {this.color});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(label, style: Theme.of(context).textTheme.labelSmall),
        const SizedBox(height: 2),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: color ?? Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Text(
            value,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              color: color != null
                  ? (color!.computeLuminance() > 0.5 ? Colors.black : Colors.white)
                  : null,
            ),
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Fermentables Tab
// ---------------------------------------------------------------------------

class _FermentablesTab extends StatelessWidget {
  final RecipeType recipe;
  const _FermentablesTab({required this.recipe});

  @override
  Widget build(BuildContext context) {
    final additions = recipe.ingredients.fermentableAdditions;
    if (additions.isEmpty) {
      return const Center(child: Text('No fermentables added.'));
    }
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: additions.length,
      itemBuilder: (context, i) {
        final f = additions[i];
        return _FermentableCard(addition: f);
      },
    );
  }
}

class _FermentableCard extends StatelessWidget {
  final FermentableAdditionType addition;
  const _FermentableCard({required this.addition});

  @override
  Widget build(BuildContext context) {
    final amount = _formatFermentableAmount(addition.amount);
    final colorStr = '${addition.color.value.toStringAsFixed(0)} ${_colorUnitLabel(addition.color.unit)}';

    return Card(
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: _srmColor(addition.color.value),
          child: const Icon(Icons.grass, color: Colors.white, size: 18),
        ),
        title: Text(addition.name),
        subtitle: Text([amount, colorStr].join(' · ')),
        trailing: Chip(label: Text(addition.type.name, style: const TextStyle(fontSize: 11))),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Hops Tab
// ---------------------------------------------------------------------------

class _HopsTab extends StatelessWidget {
  final RecipeType recipe;
  const _HopsTab({required this.recipe});

  @override
  Widget build(BuildContext context) {
    final additions = recipe.ingredients.hopAdditions;
    if (additions.isEmpty) {
      return const Center(child: Text('No hops added.'));
    }
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: additions.length,
      itemBuilder: (context, i) {
        final h = additions[i];
        return _HopCard(addition: h);
      },
    );
  }
}

class _HopCard extends StatelessWidget {
  final HopAdditionType addition;
  const _HopCard({required this.addition});

  @override
  Widget build(BuildContext context) {
    final amount = _formatHopAmount(addition.amount);
    final timing = _formatTiming(addition.timing);

    return Card(
      child: ListTile(
        leading: const CircleAvatar(
          backgroundColor: Colors.green,
          child: Icon(Icons.eco, color: Colors.white, size: 18),
        ),
        title: Text(addition.name),
        subtitle: Text([amount, if (timing.isNotEmpty) timing].join(' · ')),
        trailing: Text('${addition.alphaAcid.value.toStringAsFixed(1)}% AA'),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Cultures Tab
// ---------------------------------------------------------------------------

class _CulturesTab extends StatelessWidget {
  final RecipeType recipe;
  const _CulturesTab({required this.recipe});

  @override
  Widget build(BuildContext context) {
    final additions = recipe.ingredients.cultureAdditions;
    if (additions.isEmpty) {
      return const Center(child: Text('No cultures/yeast added.'));
    }
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: additions.length,
      itemBuilder: (context, i) {
        final c = additions[i];
        return _CultureCard(addition: c);
      },
    );
  }
}

class _CultureCard extends StatelessWidget {
  final CultureAdditionType addition;
  const _CultureCard({required this.addition});

  @override
  Widget build(BuildContext context) {
    final parts = <String>[];
    if (addition.producer != null) parts.add(addition.producer!);
    if (addition.productId != null) parts.add(addition.productId!);

    return Card(
      child: ListTile(
        leading: const CircleAvatar(
          backgroundColor: Colors.purple,
          child: Icon(Icons.science, color: Colors.white, size: 18),
        ),
        title: Text(addition.name),
        subtitle: parts.isNotEmpty ? Text(parts.join(' · ')) : null,
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Mash Tab
// ---------------------------------------------------------------------------

class _MashTab extends StatelessWidget {
  final RecipeType recipe;
  const _MashTab({required this.recipe});

  @override
  Widget build(BuildContext context) {
    final mash = recipe.mash;
    if (mash == null) {
      return const Center(child: Text('No mash schedule defined.'));
    }
    final steps = mash.mashSteps;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Text(mash.name, style: Theme.of(context).textTheme.titleMedium),
        ),
        ...steps.asMap().entries.map((entry) {
          final i = entry.key;
          final step = entry.value;
          return Card(
            child: ListTile(
              leading: CircleAvatar(child: Text('${i + 1}')),
              title: Text(step.name),
              subtitle: Text(_formatMashStep(step)),
            ),
          );
        }),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

String _formatFermentableAmount(FermentableAdditionTypeAmount? amount) {
  if (amount == null) return '';
  return switch (amount) {
    FermentableAdditionTypeAmount_VolumeType(field0: final v) =>
        '${v.value} ${volumeUnitLabel(v.unit)}',
    FermentableAdditionTypeAmount_MassType(field0: final m) =>
        '${m.value} ${_massUnitLabel(m.unit)}',
  };
}

String _formatHopAmount(HopAdditionTypeAmount? amount) {
  if (amount == null) return '';
  return switch (amount) {
    HopAdditionTypeAmount_VolumeType(field0: final v) =>
        '${v.value} ${volumeUnitLabel(v.unit)}',
    HopAdditionTypeAmount_MassType(field0: final m) =>
        '${m.value} ${_massUnitLabel(m.unit)}',
  };
}

String _formatTiming(TimingType timing) {
  final parts = <String>[];
  if (timing.use != null) parts.add(timing.use!.name);
  if (timing.time != null) {
    parts.add('${timing.time!.value.toStringAsFixed(0)} ${_timeUnitLabel(timing.time!.unit)}');
  }
  return parts.join(' · ');
}

String _formatMashStep(MashStepType step) {
  final parts = <String>[
    '${step.stepTemperature.value.toStringAsFixed(0)}°${_tempUnitLabel(step.stepTemperature.unit)}',
    '${step.stepTime.value.toStringAsFixed(0)} ${_timeUnitLabel(step.stepTime.unit)}',
    step.type.name,
  ];
  return parts.join(' · ');
}

String _massUnitLabel(MassUnitType unit) {
  return switch (unit) {
    MassUnitType.mg => 'mg',
    MassUnitType.g => 'g',
    MassUnitType.kg => 'kg',
    MassUnitType.lb => 'lb',
    MassUnitType.oz => 'oz',
  };
}

String _timeUnitLabel(TimeUnitType unit) {
  return switch (unit) {
    TimeUnitType.sec => 'sec',
    TimeUnitType.min => 'min',
    TimeUnitType.hr => 'hr',
    TimeUnitType.day => 'days',
    TimeUnitType.week => 'weeks',
  };
}

String _tempUnitLabel(TemperatureUnitType unit) {
  return switch (unit) {
    TemperatureUnitType.c => 'C',
    TemperatureUnitType.f => 'F',
  };
}

String _colorUnitLabel(ColorUnitType unit) {
  return switch (unit) {
    ColorUnitType.srm => 'SRM',
    ColorUnitType.ebc => 'EBC',
    ColorUnitType.lovi => 'Lovi',
  };
}

Color _srmColor(double srm) {
  // Approximate SRM to color mapping
  if (srm <= 2) return const Color(0xFFF8F4B4);
  if (srm <= 4) return const Color(0xFFD5BC26);
  if (srm <= 8) return const Color(0xFFC17D00);
  if (srm <= 15) return const Color(0xFF8D4000);
  if (srm <= 25) return const Color(0xFF5A1E00);
  return const Color(0xFF261716);
}
