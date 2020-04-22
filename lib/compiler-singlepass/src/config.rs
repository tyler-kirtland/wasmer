// Allow unused imports while developing
#![allow(unused_imports, dead_code)]

use crate::compiler::SinglepassCompiler;
use wasmer_compiler::{Compiler, CompilerConfig, CpuFeature, Features, Target};

#[derive(Clone)]
pub struct SinglepassConfig {
    /// Enable NaN canonicalization.
    ///
    /// NaN canonicalization is useful when trying to run WebAssembly
    /// deterministically across different architectures.
    pub enable_nan_canonicalization: bool,

    features: Features,
    target: Target,
}

impl SinglepassConfig {
    /// Creates a new configuration object with the default configuration
    /// specified.
    pub fn new() -> Self {
        Self {
            enable_nan_canonicalization: true,
            features: Default::default(),
            target: Default::default(),
        }
    }
}

impl CompilerConfig for SinglepassConfig {
    /// Gets the WebAssembly features
    fn features(&self) -> &Features {
        &self.features
    }

    /// Gets the WebAssembly features, mutable
    fn features_mut(&mut self) -> &mut Features {
        &mut self.features
    }

    /// Gets the target that we will use for compiling
    /// the WebAssembly module
    fn target(&self) -> &Target {
        &self.target
    }

    /// Gets the target that we will use for compiling
    /// the WebAssembly module, mutable
    fn target_mut(&mut self) -> &mut Target {
        &mut self.target
    }

    /// Transform it into the compiler
    fn compiler(&self) -> Box<dyn Compiler> {
        Box::new(SinglepassCompiler::new(&self))
    }
}

impl Default for SinglepassConfig {
    fn default() -> SinglepassConfig {
        SinglepassConfig::new()
    }
}
