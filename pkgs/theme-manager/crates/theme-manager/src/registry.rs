use anyhow::{Result, bail};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use theme_core::{Artifact, Config, ExtensionsConfig, PaletteProvider, ResolvedTheme, Target};
use theme_extension_luau::{ExtensionDescriptor, ExtensionKind, discover};

type ProviderFactory = Arc<dyn Fn(&toml::Value) -> Result<Box<dyn PaletteProvider>> + Send + Sync>;
type TargetFactory = Arc<dyn Fn(&toml::Value) -> Result<Option<Box<dyn Target>>> + Send + Sync>;

struct NamedTarget {
    id: String,
    inner: Box<dyn Target>,
}

impl Target for NamedTarget {
    fn id(&self) -> &str {
        &self.id
    }

    fn output_paths(&self) -> Result<Vec<PathBuf>> {
        self.inner.output_paths()
    }

    fn render(&self, theme: &ResolvedTheme) -> Result<Vec<Artifact>> {
        self.inner.render(theme)
    }
}

pub struct Registry {
    providers: BTreeMap<String, ProviderFactory>,
    targets: BTreeMap<String, TargetFactory>,
    extensions: Vec<ExtensionDescriptor>,
    diagnostics: Vec<String>,
    extension_watch_paths: Vec<PathBuf>,
}

impl Registry {
    pub fn empty() -> Self {
        Self {
            providers: BTreeMap::new(),
            targets: BTreeMap::new(),
            extensions: Vec::new(),
            diagnostics: Vec::new(),
            extension_watch_paths: Vec::new(),
        }
    }

    pub fn load(config: &Config) -> Result<Self> {
        Self::load_extensions(&config.extensions)
    }

    pub fn load_extensions(extensions: &ExtensionsConfig) -> Result<Self> {
        let mut registry = Self::empty();

        let discovery = discover(extensions)?;
        registry.diagnostics = discovery.diagnostics;
        registry.extension_watch_paths = discovery.watch_paths;

        for descriptor in discovery.extensions {
            let id = descriptor.adapter_id.clone();
            if registry.providers.contains_key(&id) || registry.targets.contains_key(&id) {
                bail!("duplicate adapter id '{id}'");
            }

            match descriptor.kind {
                ExtensionKind::Provider => {
                    let captured = descriptor.clone();
                    registry.providers.insert(
                        id,
                        Arc::new(move |settings| {
                            theme_extension_luau::build_provider(captured.clone(), settings)
                        }),
                    );
                }
                ExtensionKind::Target => {
                    let captured = descriptor.clone();
                    registry.targets.insert(
                        id,
                        Arc::new(move |settings| {
                            theme_extension_luau::build_target(captured.clone(), settings)
                        }),
                    );
                }
            }
            registry.extension_watch_paths.push(descriptor.root.clone());
            registry.extensions.push(descriptor);
        }

        registry.extension_watch_paths.sort();
        registry.extension_watch_paths.dedup();
        Ok(registry)
    }

    pub fn build_provider(
        &self,
        kind: &str,
        settings: &toml::Value,
    ) -> Result<Box<dyn PaletteProvider>> {
        let Some(factory) = self.providers.get(kind) else {
            bail!(
                "unknown provider '{kind}'. Available: {}{}",
                self.providers
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                diagnostics_suffix(&self.diagnostics)
            );
        };
        factory(settings)
    }

    pub fn build_target(
        &self,
        instance: &str,
        adapter: &str,
        settings: &toml::Value,
    ) -> Result<Option<Box<dyn Target>>> {
        let Some(factory) = self.targets.get(adapter) else {
            bail!(
                "unknown target adapter '{adapter}'. Available: {}{}",
                self.targets.keys().cloned().collect::<Vec<_>>().join(", "),
                diagnostics_suffix(&self.diagnostics)
            );
        };
        Ok(factory(settings)?.map(|inner| {
            Box::new(NamedTarget {
                id: instance.to_string(),
                inner,
            }) as Box<dyn Target>
        }))
    }

    pub fn provider_ids(&self) -> impl Iterator<Item = &String> {
        self.providers.keys()
    }

    pub fn target_ids(&self) -> impl Iterator<Item = &String> {
        self.targets.keys()
    }

    pub fn extensions(&self) -> &[ExtensionDescriptor] {
        &self.extensions
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn extension_watch_paths(&self) -> &[PathBuf] {
        &self.extension_watch_paths
    }
}

fn diagnostics_suffix(diagnostics: &[String]) -> String {
    if diagnostics.is_empty() {
        String::new()
    } else {
        format!(
            ". Extension discovery also reported: {}",
            diagnostics.join("; ")
        )
    }
}
