import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'rust/api.dart' as api;
import 'rust/frb_mirrors.dart';

/// Notifier that holds and refreshes the recipe list.
class RecipeListNotifier extends AsyncNotifier<List<api.RecipeSummary>> {
  @override
  Future<List<api.RecipeSummary>> build() => api.listRecipes();

  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(() => api.listRecipes());
  }

  Future<String> createRecipe(String name) async {
    final recipe = await api.defaultRecipe(name: name, author: '');
    final id = await api.createRecipe(name: name, recipe: recipe);
    await refresh();
    return id;
  }

  Future<void> deleteRecipe(String id) async {
    await api.deleteRecipe(id: id);
    await refresh();
  }
}

final recipeListProvider =
    AsyncNotifierProvider<RecipeListNotifier, List<api.RecipeSummary>>(
        RecipeListNotifier.new);

/// Notifier that holds and refreshes the batch list.
class BatchListNotifier extends AsyncNotifier<List<api.BatchSummary>> {
  @override
  Future<List<api.BatchSummary>> build() => api.listBatches();

  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(() => api.listBatches());
  }
}

final batchListProvider =
    AsyncNotifierProvider<BatchListNotifier, List<api.BatchSummary>>(
        BatchListNotifier.new);

/// Provider for a single recipe by ID.
final recipeProvider =
    FutureProvider.family<Recipe?, String>((ref, id) => api.getRecipe(id: id));

/// Provider for recipe calculations.
final recipeCalcProvider =
    FutureProvider.family<api.RecipeCalculations, RecipeType>(
        (ref, recipe) => api.calculateRecipe(recipe: recipe));

/// Provider for equipment profiles.
final equipmentProfilesProvider =
    FutureProvider<List<EquipmentProfile>>((ref) => api.listEquipmentProfiles());

/// Provider for settings.
class SettingsNotifier extends AsyncNotifier<SettingsDocument> {
  @override
  Future<SettingsDocument> build() => api.getSettings();

  Future<void> save(SettingsDocument doc) async {
    await api.saveSettings(doc: doc);
    state = AsyncData(doc);
  }
}

final settingsProvider =
    AsyncNotifierProvider<SettingsNotifier, SettingsDocument>(
        SettingsNotifier.new);
