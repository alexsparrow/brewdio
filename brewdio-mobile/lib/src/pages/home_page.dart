import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers.dart';
import '../rust/api.dart' as api;
import '../rust/frb_mirrors.dart';
import 'recipe_editor_page.dart';

class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final recipes = ref.watch(recipeListProvider);
    final batches = ref.watch(batchListProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Brewdio')),
      body: RefreshIndicator(
        onRefresh: () async {
          await ref.read(recipeListProvider.notifier).refresh();
          await ref.read(batchListProvider.notifier).refresh();
        },
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            _SectionHeader(
              title: 'Recipes',
              trailing: IconButton(
                icon: const Icon(Icons.add),
                onPressed: () => _createRecipe(context, ref),
              ),
            ),
            recipes.when(
              data: (list) => list.isEmpty
                  ? const _EmptyState(message: 'No recipes yet. Tap + to create one.')
                  : Column(
                      children: list
                          .map((r) => _RecipeCard(recipe: r))
                          .toList(),
                    ),
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Text('Error: $e'),
            ),
            const SizedBox(height: 24),
            const _SectionHeader(title: 'Recent Batches'),
            batches.when(
              data: (list) {
                if (list.isEmpty) {
                  return const _EmptyState(message: 'No batches brewed yet.');
                }
                final sorted = [...list]
                  ..sort((a, b) => b.brewDate.compareTo(a.brewDate));
                final recent = sorted.take(6).toList();
                return Column(
                  children: recent.map((b) => _BatchCard(batch: b)).toList(),
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Text('Error: $e'),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _createRecipe(BuildContext context, WidgetRef ref) async {
    final name = await showDialog<String>(
      context: context,
      builder: (ctx) => _NameDialog(title: 'New Recipe'),
    );
    if (name == null || name.isEmpty) return;

    final id = await ref.read(recipeListProvider.notifier).createRecipe(name);
    if (!context.mounted) return;

    final recipe = await api.getRecipe(id: id);
    if (!context.mounted || recipe == null) return;

    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => RecipeEditorPage(recipe: recipe),
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;
  final Widget? trailing;

  const _SectionHeader({required this.title, this.trailing});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        children: [
          Text(title, style: Theme.of(context).textTheme.titleLarge),
          const Spacer(),
          if (trailing != null) trailing!,
        ],
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  final String message;
  const _EmptyState({required this.message});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 32),
      child: Center(
        child: Text(message, style: Theme.of(context).textTheme.bodyLarge),
      ),
    );
  }
}

class _RecipeCard extends ConsumerWidget {
  final api.RecipeSummary recipe;
  const _RecipeCard({required this.recipe});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Card(
      child: ListTile(
        title: Text(recipe.name),
        subtitle: Text(
          [
            recipe.recipeType,
            '${recipe.batchSize.value} ${volumeUnitLabel(recipe.batchSize.unit)}',
            if (recipe.styleName != null) recipe.styleName!,
          ].join(' · '),
        ),
        trailing: PopupMenuButton<String>(
          onSelected: (action) async {
            if (action == 'delete') {
              final confirm = await showDialog<bool>(
                context: context,
                builder: (ctx) => AlertDialog(
                  title: const Text('Delete Recipe'),
                  content: Text('Delete "${recipe.name}"?'),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.pop(ctx, false),
                      child: const Text('Cancel'),
                    ),
                    TextButton(
                      onPressed: () => Navigator.pop(ctx, true),
                      child: const Text('Delete'),
                    ),
                  ],
                ),
              );
              if (confirm == true) {
                await ref.read(recipeListProvider.notifier).deleteRecipe(recipe.id);
              }
            }
          },
          itemBuilder: (_) => [
            const PopupMenuItem(value: 'delete', child: Text('Delete')),
          ],
        ),
        onTap: () async {
          final full = await api.getRecipe(id: recipe.id);
          if (!context.mounted || full == null) return;
          Navigator.of(context).push(
            MaterialPageRoute(
              builder: (_) => RecipeEditorPage(recipe: full),
            ),
          );
        },
      ),
    );
  }
}

class _BatchCard extends StatelessWidget {
  final api.BatchSummary batch;
  const _BatchCard({required this.batch});

  @override
  Widget build(BuildContext context) {
    final date = DateTime.fromMillisecondsSinceEpoch(
      batch.brewDate.toInt() * 1000,
    );
    final dateStr =
        '${date.month}/${date.day}/${date.year}';

    return Card(
      child: ListTile(
        leading: const Icon(Icons.local_drink),
        title: Text(batch.name),
        subtitle: Text('${batch.recipeName} · $dateStr'),
      ),
    );
  }
}

String volumeUnitLabel(VolumeUnitType unit) {
  return switch (unit) {
    VolumeUnitType.gal => 'gal',
    VolumeUnitType.l => 'L',
    VolumeUnitType.floz => 'fl oz',
    VolumeUnitType.ml => 'mL',
    VolumeUnitType.tsp => 'tsp',
    VolumeUnitType.tbsp => 'tbsp',
    VolumeUnitType.qt => 'qt',
    VolumeUnitType.bbl => 'bbl',
    VolumeUnitType.cup => 'cup',
    VolumeUnitType.pt => 'pt',
    VolumeUnitType.ifloz => 'imp fl oz',
    VolumeUnitType.ipt => 'imp pt',
    VolumeUnitType.iqt => 'imp qt',
    VolumeUnitType.igal => 'imp gal',
    VolumeUnitType.ibbl => 'imp bbl',
  };
}

class _NameDialog extends StatefulWidget {
  final String title;
  const _NameDialog({required this.title});

  @override
  State<_NameDialog> createState() => _NameDialogState();
}

class _NameDialogState extends State<_NameDialog> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.title),
      content: TextField(
        controller: _controller,
        autofocus: true,
        decoration: const InputDecoration(hintText: 'Name'),
        onSubmitted: (_) => Navigator.pop(context, _controller.text),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context, _controller.text),
          child: const Text('Create'),
        ),
      ],
    );
  }
}
