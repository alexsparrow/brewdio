use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_json::Value;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/tags");

    let version = std::process::Command::new("git")
        .args(["describe", "--tags", "--always"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=BREWDIO_VERSION={}", version);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tarball_path = out_dir.join("beerjson.tar.gz");
    let extract_dir = out_dir.join("beerjson_extracted");

    // 1 & 2. Download and Extract (Same as before)
    let url = "https://github.com/beerjson/beerjson/archive/refs/tags/v1.0.2.tar.gz";
    let response = reqwest::blocking::get(url).expect("Failed to download beerjson");
    let mut file = File::create(&tarball_path).expect("Failed to create tarball file");
    file.write_all(&response.bytes().unwrap()).expect("Failed to write tarball");

    let tar_gz = File::open(&tarball_path).expect("Failed to read tarball");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(&extract_dir).expect("Failed to unpack tarball");

    let schema_dir = extract_dir.join("beerjson-1.0.2").join("json");
    let root_schema_path = schema_dir.join("beer.json");

    // 3. Load the root schema
    let root_content = fs::read_to_string(&root_schema_path).expect("Failed to read root schema");
    let mut root_json: Value = serde_json::from_str(&root_content).expect("Failed to parse JSON");

    // 4. BUNDLE external refs into internal definitions
    let mut bundled_defs = HashMap::new();
    bundle_refs(&mut root_json, &schema_dir, &mut bundled_defs);

    // Inject all gathered definitions into the root schema's "definitions" block
    if let Value::Object(ref mut root_map) = root_json {
        let root_defs = root_map
            .entry("definitions")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(ref mut root_defs_map) = root_defs {
            for (k, v) in bundled_defs {
                root_defs_map.insert(k, v);
            }
        }
    }

    // 5. Convert to Schemars RootSchema
    let schema: schemars::schema::RootSchema = serde_json::from_value(root_json)
        .expect("Failed to convert unified JSON to Schemars RootSchema");

    let settings = typify::TypeSpaceSettings::default();

    let mut type_space = typify::TypeSpace::new(&settings);

    type_space.add_root_schema(schema).expect("Failed to add schema to TypeSpace");

    let generated_code = type_space.to_stream();

    // 6. Format and include the necessary imports
    let syntax_tree = syn::parse2(generated_code).expect("Failed to parse generated code");
    let formatted_code = prettyplease::unparse(&syntax_tree);

    // 7. Post-process: add cfg_attr-gated derives for wasm and crdt features.
    // This ensures the same generated file works regardless of which features are active,
    // preventing stale artifact contamination between WASM and native builds.
    use regex::Regex;
    let re = Regex::new(r"(?m)^(\s*#\[derive\([^)]*\)\])").unwrap();
    let final_code = re.replace_all(
        &formatted_code,
        "$1\n#[cfg_attr(feature = \"wasm\", derive(Tsify))]\n#[cfg_attr(feature = \"wasm\", tsify(from_wasm_abi, into_wasm_abi))]\n#[cfg_attr(feature = \"crdt\", derive(autosurgeon::Hydrate, autosurgeon::Reconcile))]"
    ).to_string();

    // 7b. For newtype wrapper structs (e.g. `pub struct Foo(Bar)`), autosurgeon's derived
    // Hydrate only generates the `hydrate` entry method, not `hydrate_map`/`hydrate_string`
    // etc. This breaks when the newtype is used inside `Option<T>`, because Option dispatches
    // to those specific methods. Similarly, the derived Reconcile treats newtypes as tuple
    // structs (using key "0"), which mismatches with the transparent Hydrate delegation.
    // Remove both derives and append manual impls instead.
    let newtype_re = Regex::new(
        r"(?m)#\[cfg_attr\(feature = .crdt., derive\(autosurgeon::Hydrate, autosurgeon::Reconcile\)\)\]\n(pub struct (\w+)\((pub )?([^)]+)\);)"
    ).unwrap();

    let mut manual_impls = String::new();
    for cap in newtype_re.captures_iter(&final_code) {
        let type_name = &cap[2];
        let inner_type = &cap[4];

        // Determine which hydrate_* method to delegate based on inner type
        let extra_method = match inner_type.trim() {
            "String" => format!(
                r#"
    fn hydrate_string(s: &str) -> Result<Self, autosurgeon::HydrateError> {{
        Ok(Self(<String as autosurgeon::Hydrate>::hydrate_string(s)?))
    }}"#
            ),
            "f64" => format!(
                r#"
    fn hydrate_f64(f: f64) -> Result<Self, autosurgeon::HydrateError> {{
        Ok(Self(f))
    }}"#
            ),
            // Struct types need hydrate_map
            _ => format!(
                r#"
    fn hydrate_map<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
    ) -> Result<Self, autosurgeon::HydrateError> {{
        Ok(Self(<{inner} as autosurgeon::Hydrate>::hydrate_map(doc, obj)?))
    }}"#,
                inner = inner_type.trim()
            ),
        };

        manual_impls.push_str(&format!(
            r#"
#[cfg(feature = "crdt")]
impl autosurgeon::Reconcile for {type_name} {{
    type Key<'a> = <{inner} as autosurgeon::Reconcile>::Key<'a>;

    fn reconcile<R: autosurgeon::Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {{
        self.0.reconcile(reconciler)
    }}
}}

#[cfg(feature = "crdt")]
impl autosurgeon::Hydrate for {type_name} {{
    fn hydrate<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
        prop: autosurgeon::Prop<'_>,
    ) -> Result<Self, autosurgeon::HydrateError> {{
        Ok(Self(<{inner} as autosurgeon::Hydrate>::hydrate(doc, obj, prop)?))
    }}
{extra_method}
}}
"#,
            type_name = type_name,
            inner = inner_type.trim(),
            extra_method = extra_method,
        ));
    }

    let final_code = newtype_re.replace_all(
        &final_code,
        "$1"
    ).to_string();

    // Prepended imports so the compiler knows where derives come from
    let final_output = format!(
        r#"// DO NOT EDIT — this file is auto-generated by build.rs from the BeerJSON schema.
// Any manual changes will be overwritten on the next build.

    #[cfg(feature = "wasm")]
    use tsify::Tsify;


    {}

    // --- Manual Reconcile + Hydrate impls for newtype wrappers ---
    // autosurgeon's derive for newtypes doesn't generate the right dispatch methods.
    // Reconcile is made transparent (delegates to inner type) to match the Hydrate impls.
    {}
    "#,
        final_code,
        manual_impls
    );

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = PathBuf::from(manifest_dir).join("src").join("beerjson_types.rs");

    fs::write(&dest_path, final_output).expect("Failed to write generated Rust types");
}

/// Flattens external files by moving ALL their definitions into a master map
/// and rewriting external $refs to internal $refs.
fn bundle_refs(val: &mut Value, base_dir: &Path, defs: &mut HashMap<String, Value>) {
    match val {
        Value::Array(arr) => {
            for item in arr {
                bundle_refs(item, base_dir, defs);
            }
        }
        Value::Object(obj) => {
            if let Some(Value::String(ref_str)) = obj.remove("$ref") {
                let parts: Vec<&str> = ref_str.split('#').collect();
                let file_part = parts[0];
                let pointer_part = parts.get(1).unwrap_or(&"");

                if !file_part.is_empty() {
                    let ref_path = base_dir.join(file_part);
                    let content = fs::read_to_string(&ref_path)
                        .unwrap_or_else(|_| panic!("Missing ref file: {:?}", ref_path));
                    let mut resolved_file: Value = serde_json::from_str(&content)
                        .expect("Failed to parse ref file");

                    // 1. HARVEST ALL DEFINITIONS from the external file
                    // This ensures any nested internal refs have their targets copied over too.
                    if let Some(Value::Object(file_defs)) = resolved_file.get_mut("definitions") {
                        // Extract keys first to satisfy the borrow checker
                        let keys: Vec<String> = file_defs.keys().cloned().collect();
                        for key in keys {
                            if !defs.contains_key(&key) {
                                // Insert a null placeholder to prevent cyclic loops
                                defs.insert(key.clone(), Value::Null);

                                let mut def_val = file_defs.remove(&key).unwrap();
                                bundle_refs(&mut def_val, base_dir, defs);

                                // Overwrite placeholder with the fully resolved definition
                                defs.insert(key, def_val);
                            }
                        }
                    }

                    // 2. Figure out the exact name of the definition this $ref was pointing to
                    let target_def_name = if pointer_part.is_empty() || *pointer_part == "/" {
                        // Edge case: pointing to the whole file
                        let fallback_name = file_part.replace(".json", "");
                        if !defs.contains_key(&fallback_name) {
                            defs.insert(fallback_name.clone(), Value::Null);
                            bundle_refs(&mut resolved_file, base_dir, defs);
                            defs.insert(fallback_name.clone(), resolved_file);
                        }
                        fallback_name
                    } else {
                        // Standard case: pointing to a specific definition
                        pointer_part.split('/').last().unwrap().to_string()
                    };

                    // 3. Clear the object (JSON Schema ignores siblings of $ref anyway)
                    // and insert our new, safe internal reference.
                    obj.clear();
                    obj.insert(
                        "$ref".to_string(),
                        Value::String(format!("#/definitions/{}", target_def_name))
                    );
                } else {
                    // It was already an internal ref! Put it back.
                    // Because we are now harvesting all definitions, this target is guaranteed to exist.
                    obj.clear();
                    obj.insert("$ref".to_string(), Value::String(ref_str));
                }
            }

            // Recursively process any child objects
            for (_, v) in obj.iter_mut() {
                bundle_refs(v, base_dir, defs);
            }
        }
        _ => {}
    }
}
