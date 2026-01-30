//! Integration tests for the full agent detection pipeline.
//!
//! Tests all agent/language/framework combinations for remediation routing.

#[cfg(test)]
mod agent_routing_tests {
    use crate::detection::{detect_full, Agent, ChangedFile, Framework, Language};

    fn file(path: &str) -> ChangedFile {
        ChangedFile {
            path: path.to_string(),
            additions: 10,
            deletions: 2,
        }
    }

    // ===== 🦖 REX - Rust =====
    #[test]
    fn rex_rust_files() {
        let files = vec![file("src/main.rs"), file("src/lib.rs")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Rust));
        assert_eq!(agent, Agent::Rex);
    }

    #[test]
    fn rex_shell_scripts() {
        let files = vec![file("scripts/deploy.sh")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Shell));
        assert_eq!(agent, Agent::Rex);
    }

    #[test]
    fn rex_yaml_config() {
        let files = vec![file(".github/workflows/ci.yaml")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Yaml));
        assert_eq!(agent, Agent::Rex);
    }

    // ===== 🐻 GRIZZ - Go =====
    #[test]
    fn grizz_go_files() {
        let files = vec![file("cmd/server/main.go")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Go));
        assert_eq!(agent, Agent::Grizz);
    }

    // ===== ⭐ NOVA - TypeScript Backend =====
    #[test]
    fn nova_elysia_backend() {
        let files = vec![file("src/index.ts")];
        let pkg = r#"{"dependencies": {"elysia": "1.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::Elysia));
        assert_eq!(agent, Agent::Nova);
    }

    #[test]
    fn nova_express_backend() {
        let files = vec![file("src/app.ts")];
        let pkg = r#"{"dependencies": {"express": "4.18.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::Express));
        assert_eq!(agent, Agent::Nova);
    }

    #[test]
    fn nova_hono_backend() {
        let files = vec![file("src/index.ts")];
        let pkg = r#"{"dependencies": {"hono": "4.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::Hono));
        assert_eq!(agent, Agent::Nova);
    }

    // ===== 🔥 BLAZE - Web Frontend =====
    #[test]
    fn blaze_nextjs_web() {
        let files = vec![file("app/page.tsx")];
        let pkg = r#"{"dependencies": {"next": "15.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::NextJs));
        assert_eq!(agent, Agent::Blaze);
    }

    #[test]
    fn blaze_typescript_default() {
        let files = vec![file("src/utils.ts")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::TypeScript));
        assert_eq!(agent, Agent::Blaze); // Default TS goes to Blaze (web)
    }

    // ===== 📱 TAP - Mobile =====
    #[test]
    fn tap_expo_mobile() {
        let files = vec![file("app/(tabs)/index.tsx")];
        let pkg = r#"{"dependencies": {"expo": "52.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::Expo));
        assert_eq!(agent, Agent::Tap);
    }

    #[test]
    fn tap_swift_ios() {
        let files = vec![file("MyApp/ContentView.swift")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Swift));
        assert_eq!(agent, Agent::Tap);
    }

    #[test]
    fn tap_kotlin_android() {
        let files = vec![file("app/src/main/MainActivity.kt")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Kotlin));
        assert_eq!(agent, Agent::Tap);
    }

    // ===== ⚡ SPARK - Desktop =====
    #[test]
    fn spark_electron_desktop() {
        let files = vec![file("src/main/index.ts")];
        let pkg = r#"{"dependencies": {"electron": "29.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::Electron));
        assert_eq!(agent, Agent::Spark);
    }

    #[test]
    fn spark_tauri_desktop() {
        let files = vec![file("src-tauri/src/main.rs")];
        let pkg = r#"{"dependencies": {"@tauri-apps/api": "2.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(pkg));
        assert_eq!(result.framework, Some(Framework::Tauri));
        assert_eq!(agent, Agent::Spark);
    }

    // ===== 🥽 VEX - Unity/XR =====
    #[test]
    fn vex_unity_xr() {
        let files = vec![file("Assets/Scripts/PlayerController.cs")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.framework, Some(Framework::Unity));
        assert_eq!(agent, Agent::Vex);
    }

    #[test]
    fn vex_csharp_default() {
        let files = vec![file("src/Program.cs")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::CSharp));
        assert_eq!(agent, Agent::Vex);
    }

    // ===== 🎮 FORGE - Game Engines =====
    #[test]
    fn forge_unreal_game() {
        let files = vec![file("Source/MyGame/PlayerCharacter.cpp")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.framework, Some(Framework::Unreal));
        assert_eq!(agent, Agent::Forge);
    }

    #[test]
    fn forge_cpp_default() {
        let files = vec![file("src/main.cpp")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Cpp));
        assert_eq!(agent, Agent::Forge);
    }

    // ===== 🤖 GENERIC - Unsupported Languages =====
    // These languages don't have dedicated agents yet, so they fall back to Generic/Rex

    #[test]
    fn generic_python() {
        let files = vec![file("app/main.py")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Python));
        assert_eq!(agent, Agent::Generic);
        assert_eq!(agent.display_name(), "Rex"); // Generic falls back to Rex
    }

    #[test]
    fn generic_java() {
        let files = vec![file("src/main/java/App.java")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Java));
        assert_eq!(agent, Agent::Generic);
    }

    #[test]
    fn generic_ruby() {
        let files = vec![file("app/controllers/users_controller.rb")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Ruby));
        assert_eq!(agent, Agent::Generic);
    }

    #[test]
    fn generic_php() {
        let files = vec![file("src/Controller/UserController.php")];
        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Php));
        assert_eq!(agent, Agent::Generic);
    }

    // ===== Edge Cases =====
    #[test]
    fn empty_files_returns_generic() {
        let files: Vec<ChangedFile> = vec![];
        let (_result, agent) = detect_full(&files, None);
        assert_eq!(agent, Agent::Generic);
    }

    #[test]
    fn agent_id_roundtrip() {
        let agents = [
            Agent::Rex, Agent::Grizz, Agent::Nova, Agent::Blaze,
            Agent::Tap, Agent::Spark, Agent::Vex, Agent::Forge,
        ];
        for agent in agents {
            let parsed = Agent::from_id(agent.id());
            assert_eq!(parsed, Some(agent));
        }
    }
}
