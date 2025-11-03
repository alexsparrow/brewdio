import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection, settingsCollection } from "@/db";
import { useState, useRef } from "react";
import Editor from "@monaco-editor/react";
import { Button } from "@/components/ui/button";
import { RecipeHeader } from "@/components/recipe-header";
import type { editor } from "monaco-editor";
import recipeSchema from "@beerjson/beerjson/json/recipe.json";
import measurableUnitsSchema from "@beerjson/beerjson/json/measureable_units.json";
import styleSchema from "@beerjson/beerjson/json/style.json";
import hopSchema from "@beerjson/beerjson/json/hop.json";
import fermentableSchema from "@beerjson/beerjson/json/fermentable.json";
import cultureSchema from "@beerjson/beerjson/json/culture.json";
import mashSchema from "@beerjson/beerjson/json/mash.json";
import fermentationSchema from "@beerjson/beerjson/json/fermentation.json";
import boilSchema from "@beerjson/beerjson/json/boil.json";
import packagingSchema from "@beerjson/beerjson/json/packaging.json";
import { initVimMode } from "monaco-vim";

export const Route = createFileRoute("/recipes/$recipeId_/json")({
  component: RecipeJsonEditorComponent,
});

function RecipeJsonEditorComponent() {
  const { recipeId } = Route.useParams();
  const { data: recipes, status } = useLiveQuery(recipesCollection);
  const { data: settingsData } = useLiveQuery(settingsCollection);
  const navigate = useNavigate();
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const vimModeRef = useRef<any>(null);
  const [isValid, setIsValid] = useState(true);
  const [hasChanges, setHasChanges] = useState(false);

  const recipe = recipes?.find((r) => r.id === recipeId);
  const settings = settingsData?.find((s) => s.id === "user-settings");

  if (status === "loading") {
    return <div>Loading recipe...</div>;
  }

  if (!recipe) {
    return <div>Recipe not found</div>;
  }

  const handleEditorDidMount = (editor: editor.IStandaloneCodeEditor, monaco: any) => {
    editorRef.current = editor;

    // Configure JSON schema for validation and autocomplete
    // Register all BeerJSON schemas so external $refs can be resolved
    const baseUri = "https://raw.githubusercontent.com/beerjson/beerjson/master/json";

    // The recipeSchema has RecipeType under definitions, so we need to reference it
    const schemaWithRef = {
      $schema: "http://json-schema.org/draft-07/schema#",
      $ref: "#/definitions/RecipeType",
      definitions: recipeSchema.definitions,
    };

    monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
      validate: true,
      schemas: [
        {
          uri: `${baseUri}/recipe.json`,
          fileMatch: ["**/*.json", "*"],
          schema: schemaWithRef,
        },
        {
          uri: `${baseUri}/measureable_units.json`,
          schema: measurableUnitsSchema,
        },
        {
          uri: `${baseUri}/style.json`,
          schema: styleSchema,
        },
        {
          uri: `${baseUri}/hop.json`,
          schema: hopSchema,
        },
        {
          uri: `${baseUri}/fermentable.json`,
          schema: fermentableSchema,
        },
        {
          uri: `${baseUri}/culture.json`,
          schema: cultureSchema,
        },
        {
          uri: `${baseUri}/mash.json`,
          schema: mashSchema,
        },
        {
          uri: `${baseUri}/fermentation.json`,
          schema: fermentationSchema,
        },
        {
          uri: `${baseUri}/boil.json`,
          schema: boilSchema,
        },
        {
          uri: `${baseUri}/packaging.json`,
          schema: packagingSchema,
        },
      ],
      enableSchemaRequest: false,
      allowComments: false,
    });

    // Enable autocomplete
    monaco.languages.json.jsonDefaults.setModeConfiguration({
      documentFormattingEdits: true,
      documentRangeFormattingEdits: true,
      completionItems: true,
      hovers: true,
      documentSymbols: true,
      tokens: true,
      colors: true,
      foldingRanges: true,
      diagnostics: true,
      selectionRanges: true,
    });

    // Enable vim mode if setting is enabled
    if (settings?.vimMode) {
      const statusNode = document.getElementById("vim-status");
      vimModeRef.current = initVimMode(editor, statusNode);
    }

    // Listen for validation changes
    const model = editor.getModel();
    if (model) {
      monaco.editor.onDidChangeMarkers(() => {
        const markers = monaco.editor.getModelMarkers({ resource: model.uri });
        const hasErrors = markers.some((marker: any) => marker.severity === monaco.MarkerSeverity.Error);
        setIsValid(!hasErrors);
      });
    }
  };

  const handleEditorChange = () => {
    setHasChanges(true);
  };

  const handleSave = async () => {
    if (!editorRef.current || !isValid) return;

    try {
      const value = editorRef.current.getValue();
      const parsedRecipe = JSON.parse(value);

      // Update the recipe in the database
      await recipesCollection.update(recipeId, (draft) => {
        draft.recipe = parsedRecipe;
        draft.updatedAt = Date.now();
      });

      setHasChanges(false);

      // Navigate back to recipe detail
      navigate({ to: "/recipes/$recipeId", params: { recipeId } });
    } catch (error) {
      console.error("Failed to save recipe:", error);
    }
  };

  const handleCancel = () => {
    if (editorRef.current && recipe) {
      // Revert to original recipe JSON
      editorRef.current.setValue(JSON.stringify(recipe.recipe, null, 2));
      setHasChanges(false);
    }
  };

  return (
    <div className="flex flex-col h-full gap-4">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <RecipeHeader recipe={recipe} showDelete={false} />
          <p className="text-sm text-muted-foreground mt-2">
            JSON Editor - Advanced editing mode
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={handleCancel} disabled={!hasChanges}>
            {hasChanges ? "Revert Changes" : "No Changes"}
          </Button>
          <Button onClick={handleSave} disabled={!isValid || !hasChanges}>
            {!isValid ? "Invalid JSON" : hasChanges ? "Save Changes" : "No Changes"}
          </Button>
        </div>
      </div>

      {!isValid && (
        <div className="bg-destructive/10 border border-destructive text-destructive px-4 py-2 rounded-md text-sm">
          The JSON contains validation errors. Fix the errors to enable saving.
        </div>
      )}

      <div className="flex-1 border rounded-lg overflow-hidden flex flex-col">
        <Editor
          height="100%"
          defaultLanguage="json"
          defaultValue={JSON.stringify(recipe.recipe, null, 2)}
          onMount={handleEditorDidMount}
          onChange={handleEditorChange}
          theme="vs-dark"
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            tabSize: 2,
            formatOnPaste: true,
            formatOnType: true,
            automaticLayout: true,
            quickSuggestions: {
              other: true,
              comments: false,
              strings: true,
            },
            suggest: {
              showProperties: true,
              showValues: true,
              showEnums: true,
            },
            suggestOnTriggerCharacters: true,
          }}
        />
        {settings?.vimMode && (
          <div
            id="vim-status"
            className="bg-muted px-3 py-1 text-xs font-mono border-t"
          />
        )}
      </div>
    </div>
  );
}
