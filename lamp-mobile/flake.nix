{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      mkAndroidEnv = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.android_sdk.accept_license = true;
            config.allowUnfree = true;
          };

          androidComposition = pkgs.androidenv.composeAndroidPackages {
            buildToolsVersions = [ "34.0.0" "35.0.0" ];
            platformVersions = [ "35" ];
            includeEmulator = false;
            includeNDK = false;
            includeSources = false;
            includeSystemImages = false;
          };
        in
        { inherit pkgs; androidSdk = androidComposition.androidsdk; };
    in
    {
      devShells = forAllSystems (system:
        let
          env = mkAndroidEnv system;
          pkgs = env.pkgs;
          androidSdk = env.androidSdk;

          # Wrapper that tells AGP to use aapt2 from Nix SDK (the Maven one
          # is dynamically linked and won't run on NixOS).
          gradleWrapper = pkgs.writeShellScriptBin "gradle" ''
            exec ${pkgs.gradle}/bin/gradle \
              "-Pandroid.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/35.0.0/aapt2" \
              "$@"
          '';

          fetchDeps = pkgs.writeShellScriptBin "lamp-fetch-deps" ''
            set -euo pipefail
            REPO="''${LAMP_MAVEN_REPO:-$HOME/.cache/lamp-mobile/maven-repo}"

            echo "Resolving Maven dependencies..."
            echo "Target: $REPO"

            export GRADLE_USER_HOME="''${GRADLE_USER_HOME:-$HOME/.gradle}"
            unset MAVEN_REPO

            # Resolve all subproject dependencies to populate the Gradle cache.
            gradle --no-daemon dependencies 2>&1 || true
            gradle --no-daemon buildEnvironment 2>&1 || true

            # Clean up verification metadata if generated.
            rm -f gradle/verification-metadata.xml

            CACHE="$GRADLE_USER_HOME/caches/modules-2/files-2.1"
            if [ ! -d "$CACHE" ]; then
              echo "Error: Gradle cache not found at $CACHE"
              exit 1
            fi

            echo "Converting Gradle cache to Maven repo layout..."
            rm -rf "$REPO"
            mkdir -p "$REPO"

            cd "$CACHE"
            find . -type f | while IFS= read -r file; do
              file="''${file#./}"
              group=$(echo "$file" | cut -d'/' -f1)
              artifact=$(echo "$file" | cut -d'/' -f2)
              version=$(echo "$file" | cut -d'/' -f3)
              filename=$(basename "$file")
              ext="''${filename##*.}"
              group_path=$(echo "$group" | tr '.' '/')
              dest="$REPO/$group_path/$artifact/$version"
              mkdir -p "$dest"

              # Gradle caches AARs with their original names (e.g. runtime-release.aar).
              # Maven repos expect $artifactId-$version.$ext naming.
              expected="$artifact-$version.$ext"
              if [ "$ext" = "aar" ] || [ "$ext" = "jar" ]; then
                if [ "$filename" != "$expected" ] && ! echo "$filename" | grep -q "^$artifact-$version"; then
                  cp "$file" "$dest/$expected"
                else
                  cp "$file" "$dest/$filename"
                fi
              else
                cp "$file" "$dest/$filename"
              fi
            done

            count=$(find "$REPO" -type f | wc -l)
            echo "Done. $count artifacts in $REPO"
          '';
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              androidSdk
              gradleWrapper
              kotlin
              jdk17
              fetchDeps
            ];

            ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
            ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
            JAVA_HOME = "${pkgs.jdk17}";
            GRADLE_OPTS = "-Dorg.gradle.daemon=false";

            shellHook = ''
              REPO="''${LAMP_MAVEN_REPO:-$HOME/.cache/lamp-mobile/maven-repo}"
              if [ -d "$REPO" ]; then
                export MAVEN_REPO="$REPO"
              else
                echo "Maven repo not found. Run: lamp-fetch-deps"
                echo "Then re-enter the shell: exit && nix develop"
              fi

            '';
          };
        }
      );
    };
}
