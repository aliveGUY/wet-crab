import subprocess
import os
import signal
import sys

# ANSI escape codes
BOLD_WHITE = "\033[1;37m"
RESET = "\033[0m"

BUILD_DIR = os.path.join(os.path.dirname(__file__), "..", "build/linux")


def info(message):
    print(f"{BOLD_WHITE}[INFO]{RESET}: {message}")


def cleanup():
    """Clean up copied files"""
    try:
        if os.path.exists("src/main.rs"):
            subprocess.run("rm src/main.rs", shell=True, check=True)
        if os.path.exists("Cargo.toml"):
            subprocess.run("rm Cargo.toml", shell=True, check=True)
        info("🧹 Cleaned up temporary files")
    except Exception as e:
        info(f"⚠️  Warning: Could not clean up files: {e}")


def signal_handler(sig, frame):
    """Handle SIGINT (Ctrl+C) signal."""
    info("🛑 Build interrupted by user (Ctrl+C).")
    cleanup()
    sys.exit(1)


def main():
    # Register the SIGINT handler
    signal.signal(signal.SIGINT, signal_handler)

    info(f"📦 Building for Linux...")
    
    try:
        # Copy the source files
        subprocess.run(f"cp {BUILD_DIR}/main.linux.rs src/main.rs", shell=True, check=True)
        subprocess.run(f"cp {BUILD_DIR}/Cargo.linux.toml Cargo.toml", shell=True, check=True)

        # Run the build process
        subprocess.run("cargo run", shell=True, check=True)

    except subprocess.CalledProcessError as e:
        info(f"❌ Build failed with return code {e.returncode}")
        cleanup()
        sys.exit(e.returncode)
    except Exception as e:
        info(f"❌ Unexpected error: {e}")
        cleanup()
        sys.exit(1)
    finally:
        # Always clean up, even on successful completion
        cleanup()


if __name__ == "__main__":
    main()
