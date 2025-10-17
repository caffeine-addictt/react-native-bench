use crate::{
    cliutil,
    fsutil::{self, OPerms},
};
use anyhow::{Context, Result};
use clap::Args;

use crate::{commands::common::CommonArgs, make_cmd};

#[derive(Args, Debug)]
pub(crate) struct AndroidBuildArgs {
    #[clap(flatten)]
    config: CommonArgs,

    /// Patches cpp-adapter.cpp
    ///
    /// Looks like react native v0.80+ removed some stuff
    /// causing cpp bindings to be out-of-date and incompatible
    /// with RN v0.80+.
    ///
    /// https://github.com/jhugman/uniffi-bindgen-react-native/issues/295
    #[clap(long = "unpatch-cpp", default_value_t = false)]
    unpatch_cpp: bool,

    /// Patches ubrn codegen & cargo-ndk v4+
    ///
    /// 1. ubrn codegen
    ///
    /// Removes formatting calls on codegen as it is unable to
    /// properly resolve prettier binary location on windowsOS.
    /// (Will not impact use on unix)
    ///
    /// https://github.com/jhugman/uniffi-bindgen-react-native/issues/302
    ///
    /// 2. cargo-ndk v4+
    ///
    /// v4 removed the --no-strip flag and upstream has not released
    /// a version fixing this
    ///
    /// https://github.com/jhugman/uniffi-bindgen-react-native/pull/304
    /// https://github.com/jhugman/uniffi-bindgen-react-native/pull/305
    #[clap(long = "unpatch-ubrn", default_value_t = false)]
    unpatch_ubrn: bool,

    /// Patches filepaths in CMakeLists.txt
    ///
    /// Filepaths on windowsOS incorrectly generates with `\`.
    /// (Will not impact use on unix)
    ///
    /// No issue upstream about this so far
    #[clap(long = "unpatch-cmake", default_value_t = false)]
    unpatch_cmake: bool,
}

impl AndroidBuildArgs {
    pub(crate) fn build(&self) -> Result<()> {
        self.config.setup()?;

        let metrics = fsutil::pwd()?.join("rust").join("metrics");
        if !metrics
            .try_exists()
            .context("failed to check metrics directory")?
        {
            anyhow::bail!("metrics directory not found");
        }

        if self.unpatch_ubrn {
            make_cmd! {
                "yarn", "ubrn", "build", "android", "--and-generate";
            }
            .run_live("building with ubrn (unpatched)")?;
        } else {
            make_cmd!(
                "uniffi-bindgen-react-native",
                "build",
                "android",
                "--and-generate";
            )
            .run_live("building with ubrn (patched)")?;
        }

        if !self.unpatch_cmake {
            let mut prog = cliutil::Step::new("patching CMakeLists.txt");
            prog.show();

            let mut f = fsutil::open(
                fsutil::pwd()?.join("android").join("CMakeLists.txt"),
                OPerms::READ | OPerms::WRITE,
            )?;
            #[cfg(windows)]
            f.lock()?;

            let mut f_content = fsutil::read(&mut f)?;
            f_content = f_content.replace("\\", "/"); // replace \ with /
            fsutil::write_over(&mut f, f_content.as_bytes())?;
            #[cfg(windows)]
            f.unlock()?;
        }

        if !self.unpatch_cpp {
            let mut prog = cliutil::Step::new("patching cpp-adapter.cpp...");
            prog.show();

            let mut f = fsutil::open(
                fsutil::pwd()?.join("android").join("cpp-adapter.cpp"),
                OPerms::READ | OPerms::WRITE,
            )?;
            #[cfg(windows)]
            f.lock()?;

            let f_content = fsutil::read(&mut f)
                .unwrap()
                .lines()
                .map(str::to_string)
                .collect::<Vec<_>>();
            let new_content = "#include <fbjni/fbjni.h>\n".to_string()
                + &f_content[..26].join("\n")
                + CPP_ADAPTER_MAIN_PATCH
                + &f_content[55..].join("\n");
            fsutil::write_over(&mut f, new_content.as_bytes())?;

            #[cfg(windows)]
            f.unlock()?;
        }

        eprintln!("done building! you can now run `kizubin run android`");
        Ok(())
    }
}

// 27-55
const CPP_ADAPTER_MAIN_PATCH: &str = "
    try {
        if (callInvokerHolderJavaObj == nullptr) {
            return false;
        }

        auto alias = facebook::jni::alias_ref<jobject>(callInvokerHolderJavaObj);
        auto holder = facebook::jni::static_ref_cast<facebook::react::CallInvokerHolder::javaobject>(alias);
        if (!holder) {
            return false;
        }

        auto jsCallInvoker = holder->cthis()->getCallInvoker();
        if (!jsCallInvoker) {
            return false;
        }

        auto runtime = reinterpret_cast<jsi::Runtime *>(rtPtr);
        return {{ ns }}::installRustCrate(*runtime, jsCallInvoker);
    } catch (...) {
        return false;
    }
";
